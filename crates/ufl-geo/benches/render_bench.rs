use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use ufl_geo::{render, GeoExpr};

fn create_complex_expr() -> GeoExpr {
    // Generate a deep tree to benchmark rendering
    let mut e = GeoExpr::Var("v".to_string());
    for _ in 0..10 {
        e = GeoExpr::Sandwich(
            Box::new(GeoExpr::Exp(Box::new(GeoExpr::Basis(14)))),
            Box::new(e),
        );
    }
    e
}

fn bench_render(c: &mut Criterion) {
    let e = create_complex_expr();
    c.bench_function("render complex", |b| b.iter(|| render(black_box(&e))));
}

criterion_group!(benches, bench_render);
criterion_main!(benches);
