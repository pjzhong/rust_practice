use crate::algorithms::Solution;

#[allow(dead_code)]
impl Solution {
    /// click [`here`] to leetcode
    ///
    /// [`here`]: https://leetcode.com/problems/product-of-array-except-self/
    #[allow(dead_code)]
    pub fn product_except_self(nums: Vec<i32>) -> Vec<i32> {
        let mut prefix_sum = Vec::from(nums.as_slice());
        let mut sufix_sum = Vec::from(nums.as_slice());
        let mut left = 1;
        let mut right = nums.len() - 1;
        for _ in 1..nums.len() {
            prefix_sum[left] *= prefix_sum[left - 1];
            sufix_sum[right - 1] *= sufix_sum[right];
            left += 1;
            right -= 1;
        }


        let mut res = vec![0; nums.len()];
        res[0] = sufix_sum[1];
        res[nums.len() - 1] = prefix_sum[nums.len() - 2];
        for i in 1..nums.len() - 1 {
            res[i] = prefix_sum[i - 1] * sufix_sum[i + 1];
        }

        res
    }
}

#[test]
fn product_of_array_except_self() {
    assert_eq!(
        vec![24, 12, 8, 6],
        Solution::product_except_self(vec![1, 2, 3, 4])
    );

    assert_eq!(
        vec![0, 0, 9, 0, 0],
        Solution::product_except_self(vec![-1, 1, 0, -3, 3])
    );
}
