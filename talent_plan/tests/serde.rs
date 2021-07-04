use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, Read, Write};

use bson::Document;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Move {
    dist: i32,
}

#[test]
fn bson_serde() -> io::Result<()> {
    {
        let mut f = File::create("serde_bson")?;
        for i in 0..1000 {
            let doc = bson::to_document(&Move { dist: i }).unwrap();
            doc.to_writer(&mut f).expect("write doc failed");
        }
    }

    {
        let f = File::open("serde_bson")?;
        let mut reader = BufReader::new(f);
        loop {
            match Document::from_reader(&mut reader) {
                Ok(d) => {
                   let m = bson::from_document::<Move>(d);
                    println!("{:?}", m);
                }
                Err(_) => break,
            }
        }
    }

    Ok(())
}

#[test]
fn bson_serde_u8() -> io::Result<()> {
    let mut f = Vec::new();
    {
        for i in 0..1000 {
            let doc = bson::to_document(&Move { dist: i }).unwrap();
            doc.to_writer(&mut f).expect("write doc failed");
        }
    }

    {
        let mut reader = f.as_slice();
        loop {
            match Document::from_reader(&mut reader) {
                Ok(d) => println!("{:?}", d),
                Err(_) => break,
            }
        }
    }

    Ok(())
}

#[test]
fn json_serde() -> io::Result<()> {
    let a = Move { dist: 1 };

    if let Ok(json) = serde_json::to_string(&a) {
        {
            let f = File::create("serde_json.txt")?;
            let mut write = BufWriter::new(f);
            write.write_all(&json.as_bytes())?;
        }

        {
            let f = File::open("serde_json.txt")?;
            let mut reader = BufReader::new(f);
            let mut buffer = String::new();

            reader.read_to_string(&mut buffer)?;

            let b: Move = serde_json::from_str(&buffer.as_str())?;

            println!("{:?}", a);
            println!("{:?}", b);
        }
    }

    Ok(())
}
