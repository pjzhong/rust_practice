use clap::{crate_version, App, Arg};

pub fn build_app() -> App<'static, 'static> {
    let app = App::new("fd")
        .version(crate_version!())
        .after_help(
            "Note: `fd -h` prints a short and concise overview while `fd --help` gives all \
                 details.",
        )

        .arg(
            Arg::with_name("pattern").help(
                "the search pattern (a regular expression, unless '--glob' is used; optional)",
            ).long_help(
                "the search pattern which is either a regular expression (default) or a glob \
                 pattern (if --glob is used). If no pattern has been specified, every entry \
                 is considered a match. If your pattern starts with a dash (-), make sure to \
                 pass '--' first, or it will be considered as a flag (fd -- '-foo').")
        )
        .arg(
            Arg::with_name("extension")
                .long("extension")
                .short("e")
                .multiple(true)
                .number_of_values(1)
                .takes_value(true)
                .value_name("ext")
                .help("Filter by file extension")
                .long_help(
                    "(Additionally) filter search results by their file extension. Multiple \
                     allowable file extensions can be specified.\n\
                     If you want to search for files without extension, \
                     you can use the regex '^[^.]+$' as a normal search pattern.",
                ),
        )
        .arg(
            Arg::with_name("file-type")
                .long("type")
                .short("t")
                .multiple(true)
                .number_of_values(1)
                .takes_value(true)
                .value_name("filetype")
                .possible_values(&[
                "f",
                "file",
                "d",
                "directory",
                "l",
                "symlink",
                "x",
                "executable",
                "e",
                "empty",
                "s",
                "socket",
                "p",
                "pipe",
            ])
                .hide_possible_values(true)
                .help(
                    "Filter by type: file (f), directory (d), symlink (l),\nexecutable (x), \
                         empty (e), socket (s), pipe (p)",
                )
                .long_help(
                    "Filter the search by type:\n  \
                       'f' or 'file':         regular files\n  \
                       'd' or 'directory':    directories\n  \
                       'l' or 'symlink':      symbolic links\n  \
                       's' or 'socket':       socket\n  \
                       'p' or 'pipe':         named pipe (FIFO)\n\n  \
                       'x' or 'executable':   executables\n  \
                       'e' or 'empty':        empty files or directories\n\n\
                     This option can be specified more than once to include multiple file types. \
                     Searching for '--type file --type symlink' will show both regular files as \
                     well as symlinks. Note that the 'executable' and 'empty' filters work differently: \
                     '--type executable' implies '--type file' by default. And '--type empty' searches \
                     for empty files and directories, unless either '--type file' or '--type directory' \
                     is specified in addition.\n\n\
                     Examples:\n  \
                       - Only search for files:\n      \
                           fd --type file …\n      \
                           fd -tf …\n  \
                       - Find both files and symlinks\n      \
                           fd --type file --type symlink …\n      \
                           fd -tf -tl …\n  \
                       - Find executable files:\n      \
                           fd --type executable\n      \
                           fd -tx\n  \
                       - Find empty files:\n      \
                           fd --type empty --type file\n      \
                           fd -te -tf\n  \
                       - Find empty directories:\n      \
                           fd --type empty --type directory\n      \
                           fd -te -td"
                ),
        )
        .arg(
            Arg::with_name("path")
                .multiple(true)
                .help("the root directory for the filesystem search (optional)")
                .long_help(
                    "The directory where the filesystem search is rooted (optional). If \
                         omitted, search the current working directory.",
                ),
        )
        .arg(
            Arg::with_name("search-path")
                .long("search-path")
                .takes_value(true)
                .conflicts_with("path")
                .multiple(true)
                .hidden_short_help(true)
                .number_of_values(1)
                .help("Provide paths to search as an alternative to the positional <path>")
                .long_help(
                    "Provide paths to search as an alternative to the positional <path> \
                         argument. Changes the usage to `fd [FLAGS/OPTIONS] --search-path <path> \
                         --search-path <path2> [<pattern>]`",
                ),
        );

    app
}
