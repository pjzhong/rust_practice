use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use exit_codes::ExitCode;

mod app;
mod error;
mod exit_codes;
mod filesystem;
mod output;
mod walk;
mod config;

fn main() -> Result<()> {
    let matches = app::build_app().get_matches_from(env::args_os());

    let current_directory = Path::new(".");
    let search_paths = extract_search_paths(&matches, current_directory)?;



    let result = walk::scan(&search_paths);
    match result {
        Ok(exit_code) => exit_code.exit(),
        Err(err) => {
            eprintln!("[fd error]: {:#}", err);
            ExitCode::GeneralError.exit();
        }
    }
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
