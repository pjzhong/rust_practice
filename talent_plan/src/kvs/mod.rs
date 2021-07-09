use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Seek, SeekFrom, Write};
use std::path;

use bson::Document;
use serde::{Deserialize, Serialize};

use error::{CliErr, Result};

pub mod error;

const NAME: &str = "kvs";

/// A in-memory Key-value store
pub struct KvStore {
    map: HashMap<String, u64>,
    file: File,
}

#[derive(Serialize, Deserialize, Debug)]
struct Record {
    opt: Opt,
    key: String,
    val: String,
}

#[derive(Serialize, Deserialize, Debug)]
enum Opt {
    Set,
    Rm,
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
            .open(kvs_path.clone())?;

        let mut read = BufReader::new(file);
        loop {
            let pos = read.stream_position()?;
            match Document::from_reader(&mut read) {
                Ok(d) => {
                    let record = bson::from_document::<Record>(d)?;
                    match record.opt {
                        Opt::Set => {
                            map.insert(record.key, pos);
                        }
                        Opt::Rm => {
                            map.remove(&record.key);
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
                let record = bson::from_document::<Record>(doc)?;
                Ok(Some(record.val))
            }
            _ => Ok(None),
        }
    }

    /// Inserts a key-value pair into the store.
    pub fn set(&mut self, key: String, val: String) -> Result<()> {
        let pos = self.file.seek(SeekFrom::End(0))?;
        let doc = bson::to_document(&Record {
            opt: Opt::Set,
            key: key.to_owned(),
            val,
        })?;
        doc.to_writer(&mut self.file)?;
        self.map.insert(key, pos);

        Ok(())
    }

    /// Removes a key from the store, returning the value at the key if the key
    /// was previously in the store.
    pub fn remove(&mut self, key: String) -> Result<String> {
        let value = self.get(key.to_owned())?;
        if value.is_none() {
            return Err(CliErr::KeyNotFound);
        }

        self.file.seek(SeekFrom::End(0))?;
        let doc = bson::to_document(&Record {
            opt: Opt::Rm,
            key: key.to_owned(),
            val: String::new(),
        })?;
        doc.to_writer(&mut self.file)?;
        self.file.flush()?;
        self.map.remove(&key);

        Ok(value.unwrap())
    }
}

impl Drop for KvStore {
    fn drop(&mut self) {
        if let Err(e) = self.file.flush() {
            eprintln!("{}", e);
        }
    }
}
