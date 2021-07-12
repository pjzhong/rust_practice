use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path;

use bson::Document;
use serde::{Deserialize, Serialize};

use error::{CliErr, Result};

pub mod error;

const NAME: &str = "kvs";
const COMPACT_THRESHOLD: u64 = 1024 * 1024;

/// A in-memory Key-value store
pub struct KvStore {
    map: HashMap<String, CommandPos>,
    file: File,
    uncompacted: u64,
}

struct CommandPos {
    pos: u64,
    len: u64,
}

#[derive(Serialize, Deserialize, Debug)]
enum Opt {
    Set { key: String, val: String },
    Rm { key: String },
}

impl KvStore {
    pub fn new() -> Result<Self> {
        KvStore::open(path::Path::new(""))
    }

    /// create a new In-memory from the file
    pub fn open(path: &path::Path) -> Result<KvStore> {
        let mut map = HashMap::new();

        let kvs_path = path.join(NAME);
        let file = OpenOptions::new()
            .read(true)
            .create(true)
            .append(true)
            .open(kvs_path)?;

        let mut read = BufReader::new(file);
        let mut uncompact = 0u64;
        let mut pos = read.stream_position()?;

        while let Ok(d) = Document::from_reader(&mut read) {
            let new_pos = read.stream_position()?;
            let len = new_pos - pos;
            let record = bson::from_document::<Opt>(d)?;
            match record {
                Opt::Set { key, .. } => {
                    if let Some(old) = map.insert(key, CommandPos { pos, len }) {
                        uncompact += old.len;
                    }
                }
                Opt::Rm { key } => {
                    map.remove(&key);
                    uncompact += len;
                }
            }
            pos = new_pos;
        }

        Ok(Self {
            map,
            file: read.into_inner(),
            uncompacted: uncompact,
        })
    }

    /// Returns the value corresponding to the key.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.map.get(&key) {
            Some(s) => {
                self.file.seek(SeekFrom::Start(s.pos))?;
                let doc = Document::from_reader(&mut self.file)?;
                let record = bson::from_document::<Opt>(doc)?;
                match record {
                    Opt::Set { key: _, val } => Ok(Some(val)),
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }

    /// Inserts a key-value pair into the store.
    pub fn set(&mut self, key: String, val: String) -> Result<()> {
        let pos = self.file.seek(SeekFrom::End(0))?;
        let doc = bson::to_document(&Opt::Set {
            key: key.to_owned(),
            val,
        })?;
        doc.to_writer(&mut self.file)?;
        let new_pos = self.file.stream_position()?;
        if let Some(old_command) = self.map.insert(
            key,
            CommandPos {
                pos,
                len: new_pos - pos,
            },
        ) {
            self.uncompacted += old_command.len;
        }

        self.compact()?;
        Ok(())
    }

    fn compact(&mut self) -> Result<()> {
        if self.uncompacted < COMPACT_THRESHOLD {
            return Ok(());
        }

        let mut buff = Vec::new();
        for (_, command) in self.map.iter_mut() {
            self.file.seek(SeekFrom::Start(command.pos))?;
            let mut buf_vec = vec![0u8; command.len as usize];
            self.file.read_exact(buf_vec.as_mut_slice())?;

            let pos = buff.len();
            buff.append(&mut buf_vec);
            command.pos = pos as u64;
        }

        self.file.set_len(0)?;
        self.file.write_all(&buff.as_slice())?;
        self.uncompacted = 0;
        Ok(())
    }

    /// Removes a key from the store, returning the value at the key if the key
    /// was previously in the store.
    pub fn remove(&mut self, key: String) -> Result<String> {
        let value = self.get(key.to_owned())?.ok_or(CliErr::KeyNotFound)?;

        self.file.seek(SeekFrom::End(0))?;
        let doc = bson::to_document(&Opt::Rm {
            key: key.to_owned(),
        })?;
        doc.to_writer(&mut self.file)?;
        if let Some(rm) = self.map.remove(&key) {
            self.uncompacted += rm.len;
        }

        Ok(value)
    }
}

impl Drop for KvStore {
    fn drop(&mut self) {
        if let Err(e) = self.file.flush() {
            eprintln!("{}", e);
        }
    }
}
