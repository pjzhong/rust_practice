use regex::bytes::RegexSet;
use crate::filetypes::FileTypes;

pub struct Config {
    /// The extension to search for. Only entries matching the extension will be included.
    ///
    /// The value (if present) will be a lowercase string without leading dots.
    pub extensions: Option<RegexSet>,

    /// The type of file to search for. If set to `None`, no file would displayed. If
    /// set to `Some(..)`, only the types that are specified are shown.
    pub file_types: Option<FileTypes>
}
