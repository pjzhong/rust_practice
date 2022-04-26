use crate::SearchDirection;
use crate::{light, LightOptions};
use termion::color;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct Row {
    len: usize,
    string: String,
    highlighting: Vec<light::Type>,
    pub is_lighted: bool,
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        if self.string.is_empty() {
            return String::new();
        }

        let end = end.min(self.string.len());
        let start = start.min(end);
        let mut result = String::new();
        let mut current_light = &light::Type::None;
        let start_highlight = format!("{}", color::Fg(current_light.color()));
        result.push_str(&start_highlight);

        for (index, grapheme) in self
            .string
            .graphemes(true)
            .enumerate()
            .skip(start)
            .take(end - start)
        {
            if let Some(c) = grapheme.chars().next() {
                let light_type = self.highlighting.get(index).unwrap_or(&light::Type::None);
                if light_type != current_light {
                    current_light = light_type;
                    let start_highlight = format!("{}", color::Fg(light_type.color()));
                    result.push_str(&start_highlight);
                }
                if c == '\t' {
                    result.push(' ');
                } else {
                    result.push(c)
                }
            }
        }

        let end_highlight = format!("{}", color::Fg(color::Reset));
        result.push_str(&end_highlight);
        result
    }

    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len() {
            self.string.push(c);
            self.len += 1;
        } else {
            let mut result: String = String::new();
            let mut length = 0;
            for (index, grapheme) in self.string.graphemes(true).enumerate() {
                length += 1;
                if index == at {
                    length += 1;
                    result.push(c);
                }
                result.push_str(grapheme);
            }

            self.len = length;
            self.string = result;
        }
    }

    pub fn append(&mut self, new: &Self) {
        self.string = format!("{}{}", self.string, new.string);
        self.len += new.len;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.string.is_empty()
    }

    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return;
        }
        let mut result = String::new();
        let mut length = 0;
        for (index, grapheme) in self.string.graphemes(true).enumerate() {
            if index != at {
                length += 1;
                result.push_str(grapheme);
            }
        }
        self.len = length;
        self.string = result;
    }

    pub fn split(&mut self, at: usize) -> Self {
        let mut row = String::new();
        let mut length = 0;
        let mut splitted_row = String::new();
        let mut splitted_length = 0;

        for (index, grapheme) in self.string.graphemes(true).enumerate() {
            if index < at {
                length += 1;
                row.push_str(grapheme);
            } else {
                splitted_length += 1;
                splitted_row.push_str(grapheme);
            }
        }

        self.string = row;
        self.len = length;
        Self {
            string: splitted_row,
            len: splitted_length,
            highlighting: Vec::new(),
            is_lighted: false
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }

    pub fn find(&self, query: &str, at: usize, direction: SearchDirection) -> Option<usize> {
        if at > self.len() || query.is_empty() {
            return None;
        }

        let (start, end) = if direction == SearchDirection::Forward {
            (at, self.len)
        } else {
            (0, at)
        };

        let substring: String = self
            .string
            .graphemes(true)
            .skip(start)
            .take(end - start)
            .collect();
        let matching_byte_index = if direction == SearchDirection::Forward {
            substring.find(query)
        } else {
            substring.rfind(query)
        };

        if let Some(matching_byte_index) = matching_byte_index {
            for (grap_index, (byte_index, _)) in substring.grapheme_indices(true).enumerate() {
                if matching_byte_index == byte_index {
                    return Some(start + grap_index);
                }
            }
        }
        None
    }

    fn highlight_match(&mut self, word: &Option<String>) {
        if let Some(word) = word {
            if word.is_empty() {
                return;
            }

            let mut index = 0;
            let count = word.graphemes(true).count();
            while let Some(search_match) = self.find(word.as_str(), index, SearchDirection::Forward)
            {
                if let Some(next_index) = search_match.checked_add(count) {
                    for i in search_match..next_index {
                        self.highlighting[i] = light::Type::Match;
                    }
                    index = next_index;
                }
            }
        }
    }

    fn highlight_str(
        &mut self,
        index: &mut usize,
        substring: &str,
        chars: &[char],
        light_type: light::Type,
    ) -> bool {
        if substring.is_empty() {
            return false;
        }

        for (sub_index, c) in substring.chars().enumerate() {
            if let Some(next_char) = chars.get(index.saturating_add(sub_index)) {
                if *next_char != c {
                    return false;
                }
            } else {
                return false;
            }
        }

        for _ in 0..substring.len() {
            self.highlighting.push(light_type);
            *index += 1;
        }

        true
    }

    fn highlight_keywords(
        &mut self,
        index: &mut usize,
        chars: &[char],
        keywords: &[String],
        light_type: light::Type,
    ) -> bool {
        if *index > 0 {
            if let Some(prev_char) = chars.get(*index - 1) {
                if !is_separator(*prev_char) {
                    return false;
                }
            }
        }

        for word in keywords {
            if *index < chars.len().saturating_sub(word.len()) {
                if let Some(next_char) = chars.get(*index + word.len()) {
                    if !is_separator(*next_char) {
                        continue;
                    }
                }
            }

            if self.highlight_str(index, word, chars, light_type) {
                return true;
            }
        }

        false
    }

    fn highlight_primary_keywords(
        &mut self,
        index: &mut usize,
        opts: &LightOptions,
        chars: &[char],
    ) -> bool {
        self.highlight_keywords(
            index,
            chars,
            opts.primary_keywords(),
            light::Type::PrimaryKeywords,
        )
    }

    fn highlight_secondary_keywords(
        &mut self,
        index: &mut usize,
        opts: &LightOptions,
        chars: &[char],
    ) -> bool {
        self.highlight_keywords(
            index,
            chars,
            opts.secondary_keywords(),
            light::Type::SecondaryKeywords,
        )
    }

    fn highlight_char(
        &mut self,
        index: &mut usize,
        opts: &LightOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.characters() && c == '\'' {
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                let closing_index = if *next_char == '\\' {
                    index.saturating_add(3)
                } else {
                    index.saturating_add(2)
                };

                if let Some(closing_char) = chars.get(closing_index) {
                    if *closing_char == '\'' {
                        for _ in 0..=closing_index.saturating_sub(*index) {
                            self.highlighting.push(light::Type::Character);
                            *index += 1;
                        }
                    }
                }
            }
        }
        false
    }

    fn highlight_comment(
        &mut self,
        index: &mut usize,
        opts: &LightOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.comments() && c == '/' && *index < chars.len() {
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                if *next_char == '/' {
                    for _ in *index..chars.len() {
                        self.highlighting.push(light::Type::Comment);
                        *index += 1;
                    }
                    return true;
                }
            }
        }
        false
    }

    fn highlight_multiline_comment(
        &mut self,
        index: &mut usize,
        opts: &LightOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.comments() && c == '/' && *index < chars.len() {
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                if *next_char == '*' {
                    let closing_index =
                        if let Some(closing_index) = self.string[*index + 2..].find("*/") {
                            *index + closing_index + 4
                        } else {
                            chars.len()
                        };

                    for _ in *index..closing_index {
                        self.highlighting.push(light::Type::MultilineComment);
                        *index += 1;
                    }

                    return true;
                }
            }
        }
        false
    }

    fn highlight_string(
        &mut self,
        index: &mut usize,
        opts: &LightOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.strings() && c == '"' {
            for _ in *index..chars.len() {
                self.highlighting.push(light::Type::String);
                *index += 1;
                if let Some(next_char) = chars.get(*index) {
                    if *next_char == '"' {
                        break;
                    }
                }
            }
            self.highlighting.push(light::Type::String);
            *index += 1;
            true
        } else {
            false
        }
    }

    fn highlight_number(
        &mut self,
        index: &mut usize,
        opts: &LightOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.numbers() && c.is_ascii_digit() {
            if *index > 0 {
                if let Some(prev_char) = chars.get(*index - 1) {
                    if !is_separator(*prev_char) {
                        return false;
                    }
                }
            }

            for _ in *index..chars.len() {
                self.highlighting.push(light::Type::Number);
                *index += 1;
                if let Some(next_char) = chars.get(*index) {
                    if *next_char != '.' && !next_char.is_ascii_digit() {
                        break;
                    }
                }
            }

            true
        } else {
            false
        }
    }

    pub fn highlight(
        &mut self,
        opts: &LightOptions,
        word: &Option<String>,
        start_with_comment: bool,
    ) -> bool {
        let chars = self.string.chars().collect::<Vec<char>>();
        if self.is_lighted && word.is_none() {
            if let Some(hl_type) = self.highlighting.last() {
                if *hl_type == light::Type::MultilineComment
                     && self.string.len() > 1
                    && self.string[self.string.len() - 2..] == *"*/" {
                    return true;
                }
            }
            return false;
        }


        self.highlighting.clear();
        let mut index = 0;
        let mut in_ml_comment = start_with_comment;
        if in_ml_comment {
            let closing_index = if let Some(closing_index) = self.string.find("*/") {
                closing_index + 2
            } else {
                chars.len()
            };

            for _ in 0..closing_index {
                self.highlighting.push(light::Type::MultilineComment);
            }

            index = closing_index;
        }
        while let Some(c) = chars.get(index) {
            if self.highlight_multiline_comment(&mut index, opts, *c, &chars) {
                in_ml_comment = true;
                continue;
            }

            in_ml_comment = false;
            if self.highlight_char(&mut index, opts, *c, &chars)
                || self.highlight_comment(&mut index, opts, *c, &chars)
                || self.highlight_primary_keywords(&mut index, opts, &chars)
                || self.highlight_secondary_keywords(&mut index, opts, &chars)
                || self.highlight_string(&mut index, opts, *c, &chars)
                || self.highlight_number(&mut index, opts, *c, &chars)
            {
                continue;
            }

            self.highlighting.push(light::Type::None);
            index += 1;
        }
        self.highlight_match(word);

        if in_ml_comment && &self.string[self.string.len().saturating_sub(2)..] != "*/" {
            return true;
        }

        self.is_lighted = true;
        false
    }
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
            highlighting: Vec::new(),
            len: slice.graphemes(true).count(),
            is_lighted: false,
        }
    }
}

fn is_separator(c: char) -> bool {
    c.is_ascii_punctuation() || c.is_ascii_whitespace()
}
