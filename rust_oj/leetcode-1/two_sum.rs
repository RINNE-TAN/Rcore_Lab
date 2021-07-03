fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
    let mut h = std::collections::HashMap::new();
    for (k, &v) in nums.iter().enumerate() {
        if let Some(&x) = h.get(&(target - v)) {
            return vec![x as i32, k as i32];
        }
        h.insert(v, k);
    }
    panic!("not found");
}
fn main() {
    println!("{:?}", two_sum(vec![3, 2, 4], 6))
}
