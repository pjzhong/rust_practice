use std::{
    collections::{hash_map::Entry, HashMap, HashSet, VecDeque},
    iter::FromIterator,
};

use super::Solution;

impl Solution {
    const DIRS: [(isize, isize); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];

    #[allow(unused)]
    pub fn find_words(board: Vec<Vec<char>>, words: Vec<String>) -> Vec<String> {
        //1.收集全部单词首字母，记录字母的开始坐标
        //2.从开始坐标开始上下左右搜索
        let n = board[0].len();
        let mut res = vec![];

        let mut starts: HashMap<&char, VecDeque<(usize, usize)>> = HashMap::new();

        for (x, row) in board.iter().enumerate() {
            for (y, col) in row.iter().enumerate() {
                match starts.entry(col) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().push_back((x, y));
                    }
                    Entry::Vacant(e) => {
                        e.insert(VecDeque::from_iter([(x, y)]));
                    }
                };
            }
        }

        for word in words {
            let chars = word.chars().collect::<Vec<_>>();
            if chars.iter().any(|c| !starts.contains_key(c)) {
                continue;
            }

            let start = match starts.get(&chars[0]) {
                Some(start) => start,
                None => continue,
            };

            for (x, y) in start {
                if Solution::search(
                    *x,
                    *y,
                    1,
                    &board,
                    &chars,
                    &mut HashSet::from_iter([x * n + y]),
                ) {
                    res.push(word);
                    break;
                }
            }
        }

        res.sort();
        res
    }

    fn search(
        x: usize,
        y: usize,
        cidx: usize,
        board: &[Vec<char>],
        chars: &[char],
        moved: &mut HashSet<usize>,
    ) -> bool {
        if chars.len() <= cidx {
            return true;
        }

        let (m, n) = (board.len(), board[0].len());

        let c = chars[cidx];
        for (lhs, rhs) in Solution::DIRS {
            let x = x.wrapping_add_signed(lhs);
            let y = y.wrapping_add_signed(rhs);
            if m <= x || n <= y || board[x][y] != c {
                continue;
            }

            let idx = x * n + y;
            if moved.contains(&idx) {
                continue;
            }

            moved.insert(idx);
            if Solution::search(x, y, cidx + 1, board, chars, moved) {
                return true;
            }
            moved.remove(&idx);
        }

        false
    }
}

#[test]
fn test1() {
    let w = vec![
        vec!['o', 'a', 'a', 'n'],
        vec!['e', 't', 'a', 'e'],
        vec!['i', 'h', 'k', 'r'],
        vec!['i', 'f', 'l', 'v'],
    ];
    assert_eq!(
        vec!["eat", "oath"],
        Solution::find_words(
            w,
            vec![
                "oath".to_string(),
                "pea".to_string(),
                "eat".to_string(),
                "rain".to_string(),
            ],
        )
    );
}

#[test]
fn test2() {
    let w = vec![vec!['a', 'b'], vec!['c', 'd']];
    assert_eq!(
        Vec::<String>::new(),
        Solution::find_words(w, vec!["abcd".to_string()])
    );
}

#[test]
fn test3() {
    let w = vec![
        vec!['a', 'c', 'e', 'f'],
        vec!['p', 'm', 'x', 'f'],
        vec!['p', 'b', 'e', 'f'],
        vec!['x', 'w', 'k', 'f'],
        vec!['e', 's', 'e', 'f'],
    ];
    assert_eq!(
        Vec::<String>::new(),
        Solution::find_words(w, vec!["apple".to_string(), "agency".to_string(),],)
    );
}

#[test]
fn test4() {
    let w = vec![
        vec!['a', 'b', 'c'],
        vec!['a', 'e', 'd'],
        vec!['a', 'f', 'g'],
    ];
    assert_eq!(
        vec!["abcdefg", "befa", "eaabcdgfa", "gfedcbaaa",],
        dbg!(Solution::find_words(
            w,
            vec![
                "abcdefg".to_string(),
                "gfedcbaaa".to_string(),
                "eaabcdgfa".to_string(),
                "befa".to_string(),
                "dgc".to_string(),
                "ade".to_string()
            ],
        ))
    );
}

#[test]
fn test5() {
    let w = vec![vec!['a', 'b', 'e'], vec!['b', 'c', 'd']];
    assert_eq!(
        vec!["abcdeb"],
        dbg!(Solution::find_words(w, vec!["abcdeb".to_string()],))
    );
}
