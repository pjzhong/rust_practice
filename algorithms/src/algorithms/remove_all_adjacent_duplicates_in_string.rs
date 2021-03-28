use crate::algorithms::Solution;

/// @link https://leetcode.com/problems/remove-all-adjacent-duplicates-in-string
#[allow(dead_code)]
impl Solution {
    pub fn remove_duplicates(s: String) -> String {
        let mut result = String::new();

        for char in s.chars() {
            match result.pop() {
                Some(c) => {
                    if c != char {
                        result.push(c);
                        result.push(char);
                    }
                }
                _ => result.push(char),
            }
        }

        result
    }
}

#[test]
fn remove_duplicates_test() {
    println!("{}", Solution::remove_duplicates(String::from("abbaca")));
}
