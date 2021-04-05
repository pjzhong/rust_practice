use std::collections::BinaryHeap;

use crate::algorithms::Solution;

/// @link https://leetcode.com/problems/last-stone-weight/
#[allow(dead_code)]
impl Solution {
    pub fn last_stone_weight(stones: Vec<i32>) -> i32 {
        let mut heap = BinaryHeap::from(stones);

        while 1 < heap.len() {
            let left = heap.pop().unwrap() - heap.pop().unwrap();
            if 0 < left {
                heap.push(left);
            }
        }

        heap.pop().unwrap_or(0)
    }

    pub fn last_stone_weight_none_heap(stones: Vec<i32>) -> i32 {
        let mut stones = stones;
        stones.sort();

        while 1 < stones.len() {
            let left = stones.pop().unwrap() - stones.pop().unwrap();
            if 0 < left {
                match stones.binary_search(&left) {
                    Ok(idx) | Err(idx) => {
                        stones.insert(idx, left);
                    }
                };
            }
        }

        if stones.is_empty() {
            0
        } else {
            stones[0]
        }
    }
}

#[test]
fn last_stone_weight_test() {
    assert_eq!(
        Solution::last_stone_weight(vec![2, 7, 4, 1, 8, 1]),
        Solution::last_stone_weight_none_heap(vec![2, 7, 4, 1, 8, 1])
    )
}
