use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Cursor, Seek, SeekFrom, Write};
use std::path;

use bson::Document;
use serde::{Deserialize, Serialize};

use error::{CliErr, Result};

pub mod error;

const NAME: &str = "kvs";
const COMPACT_THRESHOLD: u64 = (1024 * 1024) * 10;

/// A in-memory Key-value store
pub struct KvStore {
    map: HashMap<String, u64>,
    file: File,
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
        loop {
            let pos = read.stream_position()?;
            match Document::from_reader(&mut read) {
                Ok(d) => {
                    let record = bson::from_document::<Opt>(d)?;
                    match record {
                        Opt::Set { key, .. } => {
                            map.insert(key, pos);
                        }
                        Opt::Rm { key } => {
                            map.remove(&key);
                        }
                    }
                }
                Err(_) => break,
            }
        }

        Ok(Self {
            map,
            file: read.into_inner(),
        })
    }

    /// Returns the value corresponding to the key.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.map.get(&key) {
            Some(s) => {
                self.file.seek(SeekFrom::Start(*s))?;
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
        self.map.insert(key, pos);

        self.compact()?;
        Ok(())
    }

    fn compact(&mut self) -> Result<()> {
        let len = self.file.metadata().map_or(0, |meta| meta.len());
        if len <= COMPACT_THRESHOLD {
            return Ok(());
        }

        let mut buff = Cursor::new(Vec::new());
        let mut new_map = HashMap::new();
        for (key, pos) in self.map.to_owned() {
            self.file.seek(SeekFrom::Start(pos))?;
            let doc = Document::from_reader(&mut self.file)?;

            let pos = buff.stream_position()?;
            doc.to_writer(&mut buff)?;
            new_map.insert(key, pos);
        }

        self.file.set_len(0)?;
        self.file.write_all(&buff.get_ref().as_slice())?;
        self.map = new_map;
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
        self.map.remove(&key);

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
