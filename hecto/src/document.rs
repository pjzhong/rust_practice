use crate::Position;
use crate::Row;
use std::fs;
use std::io::{Error, Write};

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, Error> {
        let contents = fs::read_to_string(filename)?;

        Ok(Self {
            rows: contents.lines().map(Row::from).collect::<Vec<_>>(),
            file_name: Some(filename.to_string()),
        })
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    fn insert_newline(&mut self, at: &Position) {
        if at.y > self.len() {
            return;
        }

        if at.y == self.len() {
            self.rows.push(Row::default());
        } else if let Some(row) = self.rows.get_mut(at.y) {
            let new_row = row.split(at.x);
            self.rows.insert(at.y + 1, new_row);
        }
    }

    pub fn insert(&mut self, at: &Position, c: char) {
        if c == '\n' {
            self.insert_newline(at);
            return;
        }

        if at.y == self.len() {
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
        } else if at.y < self.len() {
            if let Some(row) = self.rows.get_mut(at.y) {
                row.insert(at.x, c);
            }
        }
    }

    pub fn delete(&mut self, at: &Position) {
        let len = self.len();
        if at.y >= len {
            return;
        }

        let cur_row_len = if let Some(row) = self.rows.get(at.y) {
            row.len()
        } else {
            0
        };

        //it move the cursor from the begining of current line
        //to the end of previous line before delee
        if at.x == cur_row_len && at.y + 1 < len {
            let next_row = self.rows.remove(at.y + 1);
            if let Some(row) = self.rows.get_mut(at.y) {
                row.append(&next_row);
            }
        } else if let Some(row) = self.rows.get_mut(at.y) {
            row.delete(at.x);
        }
    }

    pub fn save(&self) -> Result<(), Error> {
        if let Some(file_name) = &self.file_name {
            let mut file = fs::File::create(file_name)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
        }
        Ok(())
    }
}
