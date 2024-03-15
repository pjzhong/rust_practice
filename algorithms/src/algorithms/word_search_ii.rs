use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
};

use super::Solution;

#[derive(Default, Debug)]
struct Node {
    word: Option<usize>,
    child: HashMap<char, Node>,
}

impl Solution {
    const DIRS: [(isize, isize); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];

    /// click [`here`] to leetcode
    ///
    /// [`here`]: https://leetcode.cn/problems/word-search-ii/description/
    #[allow(unused)]
    pub fn find_words(board: Vec<Vec<char>>, words: Vec<String>) -> Vec<String> {
        //前缀树
        //深度搜索(回溯法)
        let mut tree = {
            let mut parent = Node::default();
            for (idx, word) in words.iter().enumerate() {
                let mut node = &mut parent;
                for c in word.chars() {
                    let child = node.child.entry(c).or_default();
                    node = child;
                }
                node.word = Some(idx);
            }

            parent
        };

        let n = board[0].len();
        let mut res = vec![];
        for (x, row) in board.iter().enumerate() {
            for (y, col) in row.iter().enumerate() {
                let node = match tree.child.get_mut(col) {
                    Some(node) => node,
                    None => continue,
                };

                Solution::search(
                    x,
                    y,
                    node,
                    &board,
                    &mut HashSet::from_iter([x * n + y]),
                    &mut res,
                );
            }
        }

        let mut res = res
            .iter()
            .map(|idx| words[*idx].clone())
            .collect::<Vec<_>>();
        res.sort();
        res
    }

    fn search(
        x: usize,
        y: usize,
        tree: &mut Node,
        board: &[Vec<char>],
        moved: &mut HashSet<usize>,
        res: &mut Vec<usize>,
    ) {
        if let Some(word) = tree.word.take() {
            res.push(word);
        }

        let (m, n) = (board.len(), board[0].len());
        for (lhs, rhs) in Solution::DIRS {
            let nx = x.wrapping_add_signed(lhs);
            let ny = y.wrapping_add_signed(rhs);
            if m <= nx || n <= ny {
                continue;
            }

            let idx = nx * n + ny;
            if moved.contains(&idx) {
                continue;
            }

            let c = board[nx][ny];
            let child = match tree.child.get_mut(&c) {
                Some(child) => child,
                None => continue,
            };

            moved.insert(idx);
            Solution::search(nx, ny, child, board, moved, res);
            moved.remove(&idx);
        }
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
        Solution::find_words(
            w,
            vec![
                "abcdefg".to_string(),
                "gfedcbaaa".to_string(),
                "eaabcdgfa".to_string(),
                "befa".to_string(),
                "dgc".to_string(),
                "ade".to_string()
            ],
        )
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
