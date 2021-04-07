use crate::algorithms::Solution;
use std::collections::HashMap;

/// click [`here`] to leetcode
/// [here]: https://leetcode.com/problems/sort-characters-by-frequency/
#[allow(dead_code)]
impl Solution {
    pub fn frequency_sort(s: String) -> String {
        let mut map = HashMap::new();
        for c in s.chars() {
            let counter = map.entry(c).or_insert(0);
            *counter += 1;
        }

        let mut vec = Vec::with_capacity(map.len());
        for (k, v) in map.into_iter() {
            vec.push((v, k));
        }
        vec.sort_by(|a, b| b.cmp(a));

        let mut res = String::with_capacity(s.capacity());
        for (x, y) in vec {
            for _ in 0..x {
                res.push(y);
            }
        }

        res
    }
}

#[test]
fn frequency_sort_test() {
    println!("{}", Solution::frequency_sort(String::from("abbaca")));
}
