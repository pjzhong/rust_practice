use crate::algorithms::Solution;

/// @link https://leetcode.com/problems/maximum-units-on-a-truck/
impl Solution {
    pub fn maximum_units(box_types: Vec<Vec<i32>>, truck_size: i32) -> i32 {
        let mut box_types = box_types;
        box_types.sort_by(|a, b| b[1].partial_cmp(&a[1]).unwrap());

        let mut result = 0;
        let mut truck_size = truck_size;
        for vec in box_types {
            result += truck_size.min(vec[0]) * vec[1];
            truck_size -= vec[0];
            if truck_size <= 0 {
                break;
            }
        }

        result
    }
}

#[test]
fn num_identical_pairs_test() {
    assert_eq!(
        8,
        Solution::maximum_units(vec![vec![1, 3], vec![2, 2], vec![3, 1]], 4)
    );
    assert_eq!(
        91,
        Solution::maximum_units(vec![vec![5, 10], vec![2, 5], vec![4, 7], vec![3, 9]], 10)
    );
}
