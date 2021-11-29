use clap::{crate_version, App, Arg};

pub fn build_app() -> App<'static, 'static> {
    let app = App::new("fd")
        .version(crate_version!())
        .after_help(
            "Note: `fd -h` prints a short and concise overview while `fd --help` gives all \
                 details.",
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
        );

    app
}
