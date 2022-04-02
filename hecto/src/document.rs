use crate::{FileType, Position, Row, SearchDirection};
use std::fs;
use std::io::{Error, Write};

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    dirty: bool,
    file_type: FileType,
    pub file_name: Option<String>,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, Error> {
        let contents = fs::read_to_string(filename)?;
        let file_type: FileType = filename.into();
        let rows = contents
            .lines()
            .map(|l| {
                let mut row = Row::from(l);
                row.highlight(file_type.light_options(), None);
                row
            })
            .collect::<Vec<Row>>();

        Ok(Self {
            rows,
            dirty: false,
            file_type,
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
        if at.y == self.len() {
            self.rows.push(Row::default());
        } else if let Some(row) = self.rows.get_mut(at.y) {
            let mut new_row = row.split(at.x);
            row.highlight(self.file_type.light_options(), None);
            new_row.highlight(self.file_type.light_options(), None);
            self.rows.insert(at.y + 1, new_row);
        }
    }

    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y > self.len() {
            return;
        }

        self.dirty = true;
        if c == '\n' {
            self.insert_newline(at);
            return;
        }

        if at.y == self.len() {
            let mut row = Row::default();
            row.insert(0, c);
            row.highlight(self.file_type.light_options(), None);
            self.rows.push(row);
        } else if let Some(row) = self.rows.get_mut(at.y) {
            row.insert(at.x, c);
            row.highlight(self.file_type.light_options(), None);
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
                row.highlight(self.file_type.light_options(), None);
            }
        } else if let Some(row) = self.rows.get_mut(at.y) {
            row.delete(at.x);
            row.highlight(self.file_type.light_options(), None);
        }
    }

    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(file_name) = &self.file_name {
            let mut file = fs::File::create(file_name)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            self.file_type = file_name.as_str().into();
            self.dirty = false;
        }
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn find(&self, query: &str, at: &Position, direction: SearchDirection) -> Option<Position> {
        if at.y >= self.rows.len() {
            return None;
        }

        let mut position = at.clone();

        let start = if direction == SearchDirection::Forward {
            at.y
        } else {
            0
        };

        let end = if direction == SearchDirection::Forward {
            self.rows.len()
        } else {
            at.y.saturating_add(1)
        };

        for _ in start..end {
            if let Some(row) = self.rows.get(position.y) {
                if let Some(x) = row.find(query, position.x, direction) {
                    position.x = x;
                    return Some(position);
                }

                if direction == SearchDirection::Forward {
                    position.y = position.y.saturating_add(1);
                    position.x = 0;
                } else {
                    position.y = position.y.saturating_sub(1);
                    position.x = self.rows[position.y].len();
                }
            } else {
                return None;
            }
        }
        None
    }

    pub fn highlight(&mut self, word: Option<&str>) {
        for row in &mut self.rows {
            row.highlight(self.file_type.light_options(), word);
        }
    }

    pub fn file_type(&self) -> String {
        self.file_type.name()
    }
}
