mod document;
mod editor;
mod row;
mod terminal;

pub use document::Document;
pub use editor::Position;
pub use row::Row;
pub use terminal::Terminal;

use std::default::Default;

fn main() {
    let mut editor = editor::Editor::default();

    editor.run();
}
