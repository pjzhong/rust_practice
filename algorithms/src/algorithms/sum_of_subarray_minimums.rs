use crate::algorithms::Solution;
use std::collections::LinkedList;

#[allow(dead_code)]
impl Solution {
    /// click [`here`] to leetcode
    /// [here]: https://leetcode.com/problems/sum-of-subarray-minimums/
    pub fn sum_subarray_minimums(arr: Vec<i32>) -> i32 {
        let mut res: usize = 0;
        let stack = LinkedList::new();

        for i in 0..arr.len() {
            let mut min = arr[i];
            for j in i..arr.len() {
                min = min.min(arr[j]);
                res += min as usize;
            }
        }

        (res % 1_000_000_007) as i32
    }
}

#[test]
fn sum_subarray_minimums_test() {
    assert_eq!(17, Solution::sum_subarray_minimums(vec![3, 1, 2, 4]));
    assert_eq!(
        444,
        Solution::sum_subarray_minimums(vec![11, 81, 94, 43, 3])
    );
}
