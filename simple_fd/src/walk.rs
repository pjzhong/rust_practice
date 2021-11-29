use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::{io, thread, time};

use anyhow::{anyhow, Result};
use ignore::overrides::OverrideBuilder;
use ignore::{Error, WalkBuilder};
use once_cell::unsync::OnceCell;

use crate::error::print_error;
use crate::exit_codes::ExitCode;
use crate::output;

/// Maximum size of the output buffer before flushing results to the console
pub const MAX_BUFFER_LENGTH: usize = 1000;
/// Default duration until output buffering switches to streaming.
pub const DEFAULT_MAX_BUFFER_TIME: time::Duration = time::Duration::from_millis(100);

/// The Worker threads can result in a valid entry having PathBuf or an error.
pub enum WorkerResult {
    Entry(PathBuf),
    Error(ignore::Error),
}

enum DirEntryInner {
    Normal(ignore::DirEntry),
    BrokenSymlink(PathBuf),
}

pub struct DirEntry {
    inner: DirEntryInner,
    metadata: OnceCell<Option<Metadata>>,
}

impl DirEntry {
    fn normal(e: ignore::DirEntry) -> Self {
        Self {
            inner: DirEntryInner::Normal(e),
            metadata: OnceCell::new(),
        }
    }

    pub fn path(&self) -> &Path {
        match &self.inner {
            DirEntryInner::Normal(e) => e.path(),
            DirEntryInner::BrokenSymlink(path_buf) => path_buf.as_path(),
        }
    }
}

pub fn scan(path_vec: &[PathBuf]) -> Result<ExitCode> {
    let mut path_iter = path_vec.iter();
    let first_path_buf = path_iter
        .next()
        .expect("Error: Path vector can not be empty");

    let (tx, rx) = mpsc::channel();

    let override_builder = OverrideBuilder::new(first_path_buf);

    let overrides = override_builder
        .build()
        .map_err(|_| anyhow!("what wrong?"))?;

    let mut walker = WalkBuilder::new(first_path_buf);
    walker.overrides(overrides).git_ignore(false);

    for path_entry in path_iter {
        walker.add(path_entry);
    }

    let wants_to_quit = Arc::new(AtomicBool::new(false));
    let parallel_walker = walker.threads(num_cpus::get()).build_parallel();
    let receiver_thread = spawn_receiver(&wants_to_quit, rx);

    spawn_sender(&wants_to_quit, parallel_walker, tx);

    let exit_code = receiver_thread.join().unwrap();

    if wants_to_quit.load(Ordering::Relaxed) {
        Ok(ExitCode::KilledBySigint)
    } else {
        Ok(exit_code)
    }
}

fn spawn_receiver(
    wants_to_quit: &Arc<AtomicBool>,
    rx: Receiver<WorkerResult>,
) -> thread::JoinHandle<ExitCode> {
    let wants_to_quit = Arc::clone(wants_to_quit);

    thread::spawn(move || {
        let start = time::Instant::now();

        let stdout = io::stdout();
        let stdout = stdout.lock();
        let mut stdout = io::BufWriter::new(stdout);

        let mut buffer = Vec::with_capacity(MAX_BUFFER_LENGTH);
        for worker_result in rx {
            match worker_result {
                WorkerResult::Entry(path) => {
                    buffer.push(path);

                    if MAX_BUFFER_LENGTH < buffer.len() {
                        for path in &buffer {
                            output::print_entry(&mut stdout, path, &wants_to_quit);
                        }
                        buffer.clear();
                    }
                }
                WorkerResult::Error(err) => print_error(err.to_string()),
            }
        }

        buffer.sort();
        for path in buffer {
            output::print_entry(&mut stdout, &path, &wants_to_quit);
        }

        ExitCode::Success
    })
}

fn spawn_sender(
    wants_to_quit: &Arc<AtomicBool>,
    parallel_walker: ignore::WalkParallel,
    tx: Sender<WorkerResult>,
) {
    parallel_walker.run(|| {
        let tx_thread = tx.clone();
        let wants_to_quit = Arc::clone(wants_to_quit);

        Box::new(move |entry| {
            if wants_to_quit.load(Ordering::Relaxed) {
                return ignore::WalkState::Quit;
            }

            let entry = match entry {
                Ok(ref e) if e.depth() == 0 => {
                    return ignore::WalkState::Continue;
                }
                Ok(e) => DirEntry::normal(e),
                Err(err) => {
                    return match tx_thread.send(WorkerResult::Error(err)) {
                        Ok(_) => ignore::WalkState::Continue,
                        Err(_) => ignore::WalkState::Quit,
                    }
                }
            };

            let entry_path = entry.path();

            let send_result = tx_thread.send(WorkerResult::Entry(entry_path.to_owned()));

            if send_result.is_err() {
                return ignore::WalkState::Quit;
            }

            ignore::WalkState::Continue
        })
    });
}
