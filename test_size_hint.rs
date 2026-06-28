fn main() {
    let rank = 7;
    let iter = (0..rank).map(|i| i * 2);
    let (lower, upper) = iter.size_hint();
    println!("Size hint: {}, {:?}", lower, upper);
}
