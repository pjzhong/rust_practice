use crate::algorithms::Solution;

#[allow(dead_code)]
impl Solution {
    /// click [`here`] to leetcode
    ///
    /// [`here`]: https://leetcode.com/problems/product-of-array-except-self/
    #[allow(dead_code)]
    pub fn product_except_self(nums: Vec<i32>) -> Vec<i32> {
        let len = nums.len();
        let mut res = Vec::from(nums.as_slice());
        //calc the suffix sum first
        for i in (1..len).rev() {
            res[i - 1] *= res[i];
        }

        res[0] = res[1];
        let mut prefix_sum = nums[0];
        for i in 1..len - 1 {
            res[i] = prefix_sum * res[i + 1];
            prefix_sum *= nums[i];
        }
        res[len - 1] = prefix_sum;

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
