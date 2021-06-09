use std::collections::BinaryHeap;

use crate::algorithms::Solution;


impl Solution {

    /// click [`here`] to leetcode
    /// [here]: https://leetcode.com/problems/sum-of-subarray-minimums/
    pub fn sum_subarray_minimums(arr: Vec<i32>) -> i32 {
        let mut res: usize = 0;

        for i in 0..arr.len() {
            for j in i..arr.len() {
                let mut min = i32::max_value();
                for idx in i..=j {
                    min = arr[idx].min(min)
                }

                if min != i32::max_value() {
                    res += min as usize;
                }
            }
        }

        (res % 1_000_000_007) as i32
    }
}

#[test]
fn sum_subarray_minimums_test() {
    assert_eq!(17, Solution::sum_subarray_minimums(vec![3, 1, 2, 4]));
    assert_eq!(444, Solution::sum_subarray_minimums(vec![11,81,94,43,3]));
}
