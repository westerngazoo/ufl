use ufl_tensor::target;

#[test]
#[should_panic(expected = "target dimension overflow")]
fn target_dimension_overflow_causes_panic() {
    let _ = target(usize::MAX);
}

#[test]
#[should_panic(expected = "tensor capacity overflow")]
fn target_capacity_overflow_causes_panic() {
    // 2000^6 = 6.4e19 > usize::MAX (1.8e19 on 64-bit, 4.29e9 on 32-bit)
    let _ = target(2000);
}
