use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path;

use bson::Document;
use serde::{Deserialize, Serialize};

use error::{CliErr, Result};

pub mod error;

const NAME: &str = "kvs";

/// A in-memory Key-value store
pub struct KvStore {
    map: HashMap<String, String>,
    write: BufWriter<File>,
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
            .write(true)
            .open(kvs_path.clone())?;

        let mut read = BufReader::new(file);
        loop {
            match Document::from_reader(&mut read) {
                Ok(d) => {
                    let record = bson::from_document::<Record>(d)?;
                    match record.opt {
                        Opt::Set => {
                            map.insert(record.key, record.val);
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
            write: BufWriter::new(
                OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(kvs_path)?,
            ),
        })
    }

    /// Returns the value corresponding to the key.
    pub fn get(&self, key: String) -> Result<Option<String>> {
        match self.map.get(&key) {
            Some(s) => Ok(Some(s.clone())),
            _ => Ok(None),
        }
    }

    /// Inserts a key-value pair into the store.
    pub fn set(&mut self, key: String, val: String) -> Result<()> {
        let doc = bson::to_document(&Record {
            opt: Opt::Set,
            key: key.to_owned(),
            val: val.to_owned(),
        })?;
        doc.to_writer(&mut self.write)?;

        self.map.insert(key, val);

        Ok(())
    }

    /// Removes a key from the store, returning the value at the key if the key
    /// was previously in the store.
    pub fn remove(&mut self, key: String) -> Result<String> {
        if !self.map.contains_key(&key) {
            return Err(CliErr::KeyNotFound);
        }

        let doc = bson::to_document(&Record {
            opt: Opt::Rm,
            key: key.to_owned(),
            val: String::new(),
        })?;
        doc.to_writer(&mut self.write)?;


        Ok(self.map.remove(&key).unwrap())
    }
}

impl Drop for KvStore {
    fn drop(&mut self) {
        match self.write.flush() {
            Err(e) => eprintln!("{}", e),
            _ => {}
        }
    }
}
