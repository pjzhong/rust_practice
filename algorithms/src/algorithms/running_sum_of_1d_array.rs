use crate::algorithms::Solution;

/// @link https://leetcode.com/problems/running-sum-of-1d-array/
#[allow(dead_code)]
impl Solution {
    pub fn running_sum(nums: Vec<i32>) -> Vec<i32> {
        let mut prev = 0;

        let mut result = Vec::with_capacity(nums.len());
        for value in nums {
            prev += value;
            result.push(prev);
        }

        result
    }
}

#[test]
fn running_sum_test() {
    println!("{:?}", Solution::running_sum(vec![3, 2, 4, 6666]));
}
