use crate::filesystem;
use crate::walk;

pub struct FileTypes {
    pub files: bool,
    pub directories: bool,
    pub symlinks: bool,
    pub sockets: bool,
    pub pipes: bool,
    pub executables_only: bool,
    pub empty_only: bool,
}

impl Default for FileTypes {
    fn default() -> FileTypes {
        FileTypes {
            files: false,
            directories: false,
            symlinks: false,
            sockets: false,
            pipes: false,
            executables_only: false,
            empty_only: false,
        }
    }
}

impl FileTypes {
    pub fn files_and_dir_only() -> Self {
        Self {
            files: true,
            directories: true,
            ..Default::default()
        }
    }
}

impl FileTypes {
    pub fn type_match(&self, entry: &walk::DirEntry) -> bool {
        if let Some(ref entry_type) = entry.file_type() {
            (self.files && entry_type.is_file())
                || (self.directories && entry_type.is_dir())
                || (self.symlinks && entry_type.is_symlink())
                || (self.sockets && filesystem::is_socket(*entry_type))
                || (self.pipes && filesystem::is_pipe(*entry_type))
                || (self.executables_only
                    && entry
                        .metadata()
                        .map(|m| filesystem::is_executable(m))
                        .unwrap_or(false))
                || (self.empty_only && filesystem::is_empty(entry))
        } else {
            false
        }
    }

    pub fn should_ignore(&self, entry: &walk::DirEntry) -> bool {
        !self.type_match(entry)
    }
}
