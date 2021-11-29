use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::AtomicBool;

use crate::error::print_error;
use crate::exit_codes::ExitCode;

// TODO: this function is performance critical and can probably be optimized
pub fn print_entry<W: Write>(stdout: &mut W, entry: &Path, _wants_to_quit: &AtomicBool) {
    let path = entry;

    let r = print_entry_uncolorized(stdout, path);

    if let Err(e) = r {
        if e.kind() == ::std::io::ErrorKind::BrokenPipe {
            // Exit gracefully in case of a broken pipe (e.g. 'fd ... | head -n 3').
            ExitCode::Success.exit();
        } else {
            print_error(format!("Could not write to output: {}", e));
            ExitCode::GeneralError.exit();
        }
    }
}

fn print_entry_uncolorized_base<W: Write>(stdout: &mut W, path: &Path) -> io::Result<()> {
    let separator = "\n";
    let path_string = path.to_string_lossy();

    write!(stdout, "{}{}", path_string, separator)
}

fn print_entry_uncolorized<W: Write>(stdout: &mut W, path: &Path) -> io::Result<()> {
    print_entry_uncolorized_base(stdout, path)
}
