pub struct FileType {
    name: String,
    hl_opts: LightOptions,
}

#[derive(Default)]
pub struct LightOptions {
    numbers: bool,
    strings: bool,
    characters: bool,
    comments: bool,
}

impl LightOptions {
    pub fn numbers(&self) -> bool {
        self.numbers
    }

    pub fn strings(&self) -> bool {
        self.strings
    }

    pub fn characters(&self) -> bool {
        self.characters
    }

    pub fn comments(&self) -> bool {
        self.comments
    }
}

impl Default for FileType {
    fn default() -> Self {
        Self {
            name: String::from("No filetype"),
            hl_opts: LightOptions::default(),
        }
    }
}

impl FileType {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn light_options(&self) -> &LightOptions {
        &self.hl_opts
    }
}

impl From<&str> for FileType {
    fn from(file_name: &str) -> Self {
        if file_name.ends_with(".rs") {
            Self {
                name: String::from("rust"),
                hl_opts: LightOptions {
                    numbers: true,
                    strings: true,
                    characters: true,
                    comments: true,
                },
            }
        } else {
            Self::default()
        }
    }
}
