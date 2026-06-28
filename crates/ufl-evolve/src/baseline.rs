//! The fair-MLP Gate-2 baseline (#28 / SPEC-0011 §2.5).
//!
//! The equivariant-OOD-generalization headline compares an evolved, exact
//! `GeoExpr` against an MLP. To keep that comparison honest (the de-risk caught a
//! ~30× strawman), this finds the **smallest** MLP that reaches a target accuracy
//! on the 2-link-arm forward map — not a padded net — and quantifies its
//! **out-of-distribution collapse** (the MLP is not equivariant, so it cannot
//! extrapolate the rotation structure the geometric program captures exactly).
//!
//! Pure `std` + [`ufl_prng`] (deterministic): no external ML dependency.

use ufl_prng::SplitMix64;

/// A 2-link planar arm. Forward kinematics maps joint angles to the
/// end-effector position.
#[derive(Clone, Copy, Debug)]
pub struct ArmFk {
    /// First link length.
    pub l1: f64,
    /// Second link length.
    pub l2: f64,
}

impl ArmFk {
    /// The end-effector position `(x, y)` for joint angles `t1`, `t2`.
    pub fn forward(&self, t1: f64, t2: f64) -> (f64, f64) {
        let x = self.l1 * t1.cos() + self.l2 * (t1 + t2).cos();
        let y = self.l1 * t1.sin() + self.l2 * (t1 + t2).sin();
        (x, y)
    }

    /// `n` random `(angles, position)` pairs with both angles in `[lo, hi)`.
    fn sample(
        &self,
        n: usize,
        lo: f64,
        hi: f64,
        rng: &mut SplitMix64,
    ) -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
        let span = hi - lo;
        let mut inputs = Vec::with_capacity(n);
        let mut targets = Vec::with_capacity(n);
        for _ in 0..n {
            let t1 = lo + span * rng.f64_unit();
            let t2 = lo + span * rng.f64_unit();
            let (x, y) = self.forward(t1, t2);
            inputs.push([t1, t2]);
            targets.push([x, y]);
        }
        (inputs, targets)
    }
}

/// Training hyper-parameters. [`Default`] is the real baseline; lighter configs
/// keep the default-run tests fast.
#[derive(Clone, Copy, Debug)]
pub struct TrainConfig {
    /// Training-set size (in-distribution, `[-2, 2]²`).
    pub train: usize,
    /// Test-set size (in-distribution).
    pub test: usize,
    /// Out-of-distribution set size (`[2, 3]²`).
    pub ood: usize,
    /// Optimisation epochs.
    pub epochs: usize,
    /// Mini-batch size.
    pub batch: usize,
    /// Adam learning rate.
    pub lr: f64,
}

impl Default for TrainConfig {
    fn default() -> Self {
        Self {
            train: 4000,
            test: 1000,
            ood: 1000,
            epochs: 700,
            batch: 64,
            lr: 2e-3,
        }
    }
}

/// One trained width's error report. `params == 5 * hidden + 2`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MlpReport {
    /// Hidden-layer width.
    pub hidden: usize,
    /// Total parameter count (`5·hidden + 2`).
    pub params: usize,
    /// RMSE on the training set (original units).
    pub train_rmse: f64,
    /// RMSE on the in-distribution test set.
    pub test_rmse: f64,
    /// RMSE on the out-of-distribution set (the equivariance gap).
    pub ood_rmse: f64,
}

/// Per-column mean/std for standardisation (std floored to avoid divide-by-zero).
#[derive(Clone, Copy)]
struct Stats {
    mean: [f64; 2],
    std: [f64; 2],
}

fn fit_stats(data: &[[f64; 2]]) -> Stats {
    let n = data.len().max(1) as f64;
    let mut mean = [0.0; 2];
    for row in data {
        mean[0] += row[0];
        mean[1] += row[1];
    }
    mean[0] /= n;
    mean[1] /= n;
    let mut var = [0.0; 2];
    for row in data {
        var[0] += (row[0] - mean[0]).powi(2);
        var[1] += (row[1] - mean[1]).powi(2);
    }
    Stats {
        mean,
        std: [(var[0] / n).sqrt().max(1e-9), (var[1] / n).sqrt().max(1e-9)],
    }
}

fn standardize(s: &Stats, v: [f64; 2]) -> [f64; 2] {
    [(v[0] - s.mean[0]) / s.std[0], (v[1] - s.mean[1]) / s.std[1]]
}

/// A `2 → H (tanh) → 2 (linear)` MLP with flat weight storage.
struct Mlp {
    h: usize,
    w1: Vec<f64>, // h × 2, row-major
    b1: Vec<f64>, // h
    w2: Vec<f64>, // 2 × h, row-major
    b2: Vec<f64>, // 2
}

impl Mlp {
    fn new(h: usize, rng: &mut SplitMix64) -> Self {
        let s1 = (1.0 / 2.0_f64).sqrt();
        let s2 = (1.0 / h.max(1) as f64).sqrt();
        Self {
            h,
            w1: (0..h * 2).map(|_| rng.normal(0.0, s1)).collect(),
            b1: vec![0.0; h],
            w2: (0..2 * h).map(|_| rng.normal(0.0, s2)).collect(),
            b2: vec![0.0; 2],
        }
    }

    fn params(&self) -> usize {
        5 * self.h + 2
    }

    /// Forward pass on a standardised input; returns the hidden activations and
    /// the standardised output.
    fn forward_std(&self, x: [f64; 2]) -> (Vec<f64>, [f64; 2]) {
        let mut a1 = vec![0.0; self.h];
        for (j, aj) in a1.iter_mut().enumerate() {
            *aj = (self.b1[j] + self.w1[j * 2] * x[0] + self.w1[j * 2 + 1] * x[1]).tanh();
        }
        let mut out = [self.b2[0], self.b2[1]];
        for (i, oi) in out.iter_mut().enumerate() {
            for (j, &aj) in a1.iter().enumerate() {
                *oi += self.w2[i * self.h + j] * aj;
            }
        }
        (a1, out)
    }
}

#[allow(clippy::too_many_arguments)]
fn adam_step(
    p: &mut [f64],
    m: &mut [f64],
    v: &mut [f64],
    g: &[f64],
    inv_batch: f64,
    lr: f64,
    bias1: f64,
    bias2: f64,
) {
    let (b1, b2, eps) = (0.9_f64, 0.999_f64, 1e-8);
    for (((pk, mk), vk), &gk) in p.iter_mut().zip(m).zip(v).zip(g) {
        let grad = gk * inv_batch;
        *mk = b1 * *mk + (1.0 - b1) * grad;
        *vk = b2 * *vk + (1.0 - b2) * grad * grad;
        let mhat = *mk / bias1;
        let vhat = *vk / bias2;
        *pk -= lr * mhat / (vhat.sqrt() + eps);
    }
}

/// Train one width with an explicit config (deterministic in `seed`).
pub fn train_report_with(arm: &ArmFk, hidden: usize, seed: u64, cfg: &TrainConfig) -> MlpReport {
    let mut rng = SplitMix64::new(seed);
    let (x_tr, y_tr) = arm.sample(cfg.train, -2.0, 2.0, &mut rng);
    let (x_te, y_te) = arm.sample(cfg.test, -2.0, 2.0, &mut rng);
    let (x_ood, y_ood) = arm.sample(cfg.ood, 2.0, 3.0, &mut rng);

    let xs = fit_stats(&x_tr);
    let ys = fit_stats(&y_tr);
    let x_tr_s: Vec<[f64; 2]> = x_tr.iter().map(|v| standardize(&xs, *v)).collect();
    let y_tr_s: Vec<[f64; 2]> = y_tr.iter().map(|v| standardize(&ys, *v)).collect();

    let mut mlp = Mlp::new(hidden, &mut rng);
    let (mut m_w1, mut v_w1) = (vec![0.0; mlp.w1.len()], vec![0.0; mlp.w1.len()]);
    let (mut m_b1, mut v_b1) = (vec![0.0; mlp.b1.len()], vec![0.0; mlp.b1.len()]);
    let (mut m_w2, mut v_w2) = (vec![0.0; mlp.w2.len()], vec![0.0; mlp.w2.len()]);
    let (mut m_b2, mut v_b2) = (vec![0.0; mlp.b2.len()], vec![0.0; mlp.b2.len()]);

    let n = x_tr_s.len();
    let mut order: Vec<usize> = (0..n).collect();
    let mut t = 0i32;

    for _ in 0..cfg.epochs {
        for i in (1..n).rev() {
            order.swap(i, rng.below((i + 1) as u64) as usize);
        }
        let mut start = 0;
        while start < n {
            let end = (start + cfg.batch).min(n);
            let inv_batch = 1.0 / (end - start) as f64;
            let mut g_w1 = vec![0.0; mlp.w1.len()];
            let mut g_b1 = vec![0.0; mlp.b1.len()];
            let mut g_w2 = vec![0.0; mlp.w2.len()];
            let mut g_b2 = vec![0.0; mlp.b2.len()];
            for &s in &order[start..end] {
                let x = x_tr_s[s];
                let target = y_tr_s[s];
                let (a1, out) = mlp.forward_std(x);
                let dout = [2.0 * (out[0] - target[0]), 2.0 * (out[1] - target[1])];
                for i in 0..2 {
                    g_b2[i] += dout[i];
                    for (j, &aj) in a1.iter().enumerate() {
                        g_w2[i * mlp.h + j] += dout[i] * aj;
                    }
                }
                for (j, &aj) in a1.iter().enumerate() {
                    let da = dout[0] * mlp.w2[j] + dout[1] * mlp.w2[mlp.h + j];
                    let dpre = da * (1.0 - aj * aj);
                    g_b1[j] += dpre;
                    g_w1[j * 2] += dpre * x[0];
                    g_w1[j * 2 + 1] += dpre * x[1];
                }
            }
            t += 1;
            let (bias1, bias2) = (1.0 - 0.9_f64.powi(t), 1.0 - 0.999_f64.powi(t));
            adam_step(
                &mut mlp.w1,
                &mut m_w1,
                &mut v_w1,
                &g_w1,
                inv_batch,
                cfg.lr,
                bias1,
                bias2,
            );
            adam_step(
                &mut mlp.b1,
                &mut m_b1,
                &mut v_b1,
                &g_b1,
                inv_batch,
                cfg.lr,
                bias1,
                bias2,
            );
            adam_step(
                &mut mlp.w2,
                &mut m_w2,
                &mut v_w2,
                &g_w2,
                inv_batch,
                cfg.lr,
                bias1,
                bias2,
            );
            adam_step(
                &mut mlp.b2,
                &mut m_b2,
                &mut v_b2,
                &g_b2,
                inv_batch,
                cfg.lr,
                bias1,
                bias2,
            );
            start = end;
        }
    }

    let rmse = |inputs: &[[f64; 2]], targets: &[[f64; 2]]| -> f64 {
        let mut se = 0.0;
        for (x, y) in inputs.iter().zip(targets) {
            let (_, out_s) = mlp.forward_std(standardize(&xs, *x));
            let px = out_s[0] * ys.std[0] + ys.mean[0];
            let py = out_s[1] * ys.std[1] + ys.mean[1];
            se += (px - y[0]).powi(2) + (py - y[1]).powi(2);
        }
        (se / (inputs.len().max(1) as f64 * 2.0)).sqrt()
    };

    MlpReport {
        hidden,
        params: mlp.params(),
        train_rmse: rmse(&x_tr, &y_tr),
        test_rmse: rmse(&x_te, &y_te),
        ood_rmse: rmse(&x_ood, &y_ood),
    }
}

/// Train one width with the default (real-baseline) config.
pub fn train_report(arm: &ArmFk, hidden: usize, seed: u64) -> MlpReport {
    train_report_with(arm, hidden, seed, &TrainConfig::default())
}

/// Train each width and return one report apiece.
pub fn sweep(arm: &ArmFk, widths: &[usize], seed: u64) -> Vec<MlpReport> {
    widths.iter().map(|&h| train_report(arm, h, seed)).collect()
}

/// The fewest-parameter report whose in-distribution test RMSE meets the target
/// — the anti-strawman pick (the smallest fair MLP, not a padded one).
pub fn smallest_at(reports: &[MlpReport], target_test_rmse: f64) -> Option<&MlpReport> {
    reports
        .iter()
        .filter(|r| r.test_rmse <= target_test_rmse)
        .min_by_key(|r| r.params)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARM: ArmFk = ArmFk { l1: 1.0, l2: 0.7 };

    fn fast() -> TrainConfig {
        TrainConfig {
            train: 1500,
            test: 400,
            ood: 400,
            epochs: 200,
            batch: 64,
            lr: 3e-3,
        }
    }

    /// Forward kinematics matches the closed form.
    #[test]
    fn forward_is_correct() {
        let (x, y) = ARM.forward(0.0, 0.0);
        assert!(
            (x - 1.7).abs() < 1e-12 && y.abs() < 1e-12,
            "fully extended → (1.7, 0)"
        );
        let (x, y) = ARM.forward(0.5, -0.3);
        let ex = 0.5_f64.cos() + 0.7 * 0.2_f64.cos();
        let ey = 0.5_f64.sin() + 0.7 * 0.2_f64.sin();
        assert!((x - ex).abs() < 1e-12 && (y - ey).abs() < 1e-12);
    }

    /// Training is deterministic in the seed.
    #[test]
    fn training_is_deterministic() {
        let a = train_report_with(&ARM, 8, 42, &fast());
        let b = train_report_with(&ARM, 8, 42, &fast());
        assert_eq!(a, b);
    }

    /// More capacity fits better in-distribution; a tiny net underfits.
    #[test]
    fn capacity_helps_in_distribution() {
        let tiny = train_report_with(&ARM, 2, 1, &fast());
        let bigger = train_report_with(&ARM, 16, 1, &fast());
        assert!(
            bigger.test_rmse < tiny.test_rmse,
            "more capacity should fit better: {bigger:?} vs {tiny:?}",
        );
        assert!(
            tiny.test_rmse > 0.1,
            "H=2 should underfit, got {}",
            tiny.test_rmse
        );
    }

    /// The headline shape, in one moderate train: a SMALL fair net fits
    /// in-distribution but COLLAPSES out-of-distribution (no equivariance).
    #[test]
    fn small_net_fits_in_dist_but_collapses_ood() {
        let cfg = TrainConfig {
            train: 2500,
            test: 600,
            ood: 600,
            epochs: 350,
            batch: 64,
            lr: 2.5e-3,
        };
        let r = train_report_with(&ARM, 16, 7, &cfg);
        assert!(r.params < 100, "a fair net is small: {} params", r.params);
        assert!(
            r.test_rmse < 0.1,
            "should fit in-distribution, got {}",
            r.test_rmse
        );
        assert!(
            r.ood_rmse > 4.0 * r.test_rmse && r.ood_rmse > 0.15,
            "must collapse OOD (no equivariance): test {} vs ood {}",
            r.test_rmse,
            r.ood_rmse,
        );
    }

    /// `smallest_at` picks the fewest-parameter report meeting the bar.
    #[test]
    fn smallest_at_picks_fewest_params() {
        let reports = [
            MlpReport {
                hidden: 4,
                params: 22,
                train_rmse: 0.0,
                test_rmse: 0.20,
                ood_rmse: 0.0,
            },
            MlpReport {
                hidden: 8,
                params: 42,
                train_rmse: 0.0,
                test_rmse: 0.04,
                ood_rmse: 0.0,
            },
            MlpReport {
                hidden: 16,
                params: 82,
                train_rmse: 0.0,
                test_rmse: 0.01,
                ood_rmse: 0.0,
            },
        ];
        let pick = smallest_at(&reports, 0.05).expect("a width meets 0.05");
        assert_eq!(pick.params, 42, "smallest meeting 0.05 is H=8 (42 params)");
        assert!(smallest_at(&reports, 0.001).is_none(), "none meets 0.001");
    }

    /// The faithful de-risk reproduction (heavy; full default config + sweep).
    /// Run with: `cargo test -p ufl-evolve --release -- --ignored`.
    #[test]
    #[ignore = "heavy; run with --release -- --ignored"]
    fn fair_baseline_reproduces_derisk() {
        let reports = sweep(&ARM, &[2, 4, 6, 8, 10, 16, 32], 0);
        for r in &reports {
            eprintln!(
                "H={:>2} params={:>3} train={:.4} test={:.4} ood={:.4}",
                r.hidden, r.params, r.train_rmse, r.test_rmse, r.ood_rmse,
            );
        }
        let pick = smallest_at(&reports, 0.05).expect("some width reaches test-RMSE 0.05");
        assert!(
            pick.params < 100,
            "smallest-at-0.05 is small (anti-strawman): {pick:?}"
        );
        let best = reports
            .iter()
            .min_by(|a, b| a.test_rmse.total_cmp(&b.test_rmse))
            .expect("non-empty sweep");
        assert!(
            best.ood_rmse > 5.0 * best.test_rmse && best.ood_rmse > 0.1,
            "best in-dist model must collapse OOD: {best:?}",
        );
    }
}
