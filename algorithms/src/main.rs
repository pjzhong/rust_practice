use std::io::Write;
use std::{
    env,
    fs::{self, File},
    path::MAIN_SEPARATOR,
};

pub fn main() {
    let current_dir = env::current_dir().unwrap();
    let algorithms_dir = current_dir
        .clone()
        .join("algorithms")
        .join("src")
        .join("algorithms");

    let path_prefix = current_dir
        .clone()
        .join("algorithms/")
        .into_os_string()
        .into_string()
        .unwrap();
    let suffix = String::from(".rs");

    let mut questions = Vec::new();
    let mut dirs = Vec::new();
    dirs.push(fs::read_dir(algorithms_dir.clone()));

    while !dirs.is_empty() {
        let item = dirs.pop().unwrap();
        if let Ok(entry) = item {
            for path in entry {
                let path = path.unwrap().path();
                if path.is_file() && !path.ends_with("mod.rs") {
                    let path = String::from(path.to_str().unwrap_or(""));
                    let path_string = path
                        .chars()
                        .skip(path_prefix.len())
                        .collect::<String>()
                        .replace(MAIN_SEPARATOR, "/");

                    let index = match path_string.rfind("/") {
                        Some(idx) => idx + 1,
                        _ => 0,
                    };
                    let name = path_string
                        .chars()
                        .skip(index)
                        .take(path_string.len() - index - suffix.len())
                        .collect::<String>();

                    questions.push(format!("- [{}]({})\n", name, path_string))
                } else if path.is_dir() {
                    dirs.push(fs::read_dir(path))
                }
            }
        }
    }

    let mut read_me_file = File::create(current_dir.join("algorithms").join("README.md")).unwrap();
    let all = format!(
        "{}\n\n# Questions-{}\n\n",
        "[LeetCode:pj_zhong](https://leetcode.com/pj_zhong/)",
        questions.len()
    );
    read_me_file
        .write(all.as_bytes())
        .expect("Something wrong, check out");
    questions.sort();
    for q in questions {
        read_me_file
            .write(q.as_bytes())
            .expect("Something wrong, check out");
    }
}
