use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, Result};
use regex::bytes::{RegexBuilder, RegexSetBuilder};

use exit_codes::ExitCode;

use crate::config::Config;
use crate::filetypes::FileTypes;

mod app;
mod config;
mod error;
mod exit_codes;
mod filesystem;
mod filetypes;
mod output;
mod walk;

fn main() -> Result<()> {
    let matches = app::build_app().get_matches_from(env::args_os());

    let current_directory = Path::new(".");
    let search_paths = extract_search_paths(&matches, current_directory)?;

    let pattern = extract_search_pattern(&matches)?;
    let pattern = build_pattern_regex(pattern)?;

    let config = construct_config(&matches, &pattern)?;

    let result = walk::scan(Arc::new(config), &search_paths);
    match result {
        Ok(exit_code) => exit_code.exit(),
        Err(err) => {
            eprintln!("[fd error]: {:#}", err);
            ExitCode::GeneralError.exit();
        }
    }
}

fn extract_search_pattern<'a>(matches: &'a clap::ArgMatches) -> Result<&'a str> {
    let pattern = matches
        .value_of_os("pattern")
        .map(|p| {
            p.to_str()
                .ok_or_else(|| anyhow!("The search pattern include invalid UTF-8 sequences."))
        })
        .transpose()?
        .unwrap_or("");
    Ok(pattern)
}

fn build_pattern_regex(pattern: &str) -> Result<String> {
    Ok(String::from(pattern))
}

fn extract_search_paths(
    matches: &clap::ArgMatches,
    current_directory: &Path,
) -> Result<Vec<PathBuf>> {
    let search_path = matches
        .values_of_os("path")
        .or_else(|| matches.values_of_os("search-path"))
        .map_or_else(
            || vec![current_directory.to_path_buf()],
            |paths| {
                paths
                    .filter_map(|path| {
                        let path_buffer = PathBuf::from(path);
                        if filesystem::is_existing_directory(&path_buffer) {
                            Some(path_buffer)
                        } else {
                            None
                        }
                    })
                    .collect()
            },
        );

    if search_path.is_empty() {
        return Err(anyhow!("No valid search paths given."));
    }

    Ok(search_path)
}

fn construct_config(matches: &clap::ArgMatches, pattern: &String) -> Result<Config> {
    Ok(Config {
        extensions: matches
            .values_of("extension")
            .map(|exts| {
                let patterns = exts
                    .map(|e| e.trim_start_matches('.'))
                    .map(|e| format!(r".\.{}$", regex::escape(e)));
                RegexSetBuilder::new(patterns)
                    .case_insensitive(true)
                    .build()
            })
            .transpose()?,
        file_types: matches.values_of("file-type").map_or_else(
            || Some(FileTypes::files_and_dir_only()),
            |values| {
                let mut file_types = FileTypes::default();
                for value in values {
                    match value {
                        "f" | "file" => file_types.files = true,
                        "d" | "directory" => file_types.directories = true,
                        "l" | "symlink" => file_types.symlinks = true,
                        "x" | "executable" => {
                            file_types.executables_only = true;
                            file_types.files = true;
                        }
                        "e" | "empty" => file_types.empty_only = true,
                        "s" | "socket" => file_types.sockets = true,
                        "p" | "pipe" => file_types.pipes = true,
                        _ => unreachable!(),
                    }
                }
                Some(file_types)
            },
        ),
        regex: build_regex(pattern)?,
    })
}

fn build_regex(pattern_regex: &String) -> Result<regex::bytes::Regex> {
    RegexBuilder::new(&pattern_regex)
        .dot_matches_new_line(true)
        .build()
        .map_err(|e| {
            anyhow!(
                "{}\n\nNote: You can use the '--fixed-strings' option to search for a \
                 literal string instead of a regular expression. Alternatively, you can \
                 also use the '--glob' option to match on a glob pattern.",
                e.to_string()
            )
        })
}
