use clap::{App, AppSettings, Arg, SubCommand};

pub fn build_app() -> App<'static> {
    App::new("gt")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::ColoredHelp)
        .subcommand(SubCommand::with_name("init").about("Initialises bash-script and database."))
        .subcommand(SubCommand::with_name("ls").about("List all indexed directories."))
        .subcommand(
            SubCommand::with_name("prune").about("Removes invalid indexes in the database."),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Add directories and sub-directories to index.")
                .arg(
                    Arg::with_name("all")
                        .short('a')
                        .help("Adds all subdirectory."),
                )
                .arg(
                    Arg::with_name("recursive")
                        .short('r')
                        .takes_value(true)
                        .help("Recursively indexs."),
                ),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("Remove directories and sub-directories to index.")
                .arg(
                    Arg::with_name("all")
                        .short('a')
                        .help("Removes all subdirectory."),
                )
                .arg(
                    Arg::with_name("recursive")
                        .short('r')
                        .takes_value(true)
                        .help("Recursively removes."),
                ),
        )
        .subcommand(
            SubCommand::with_name("jump")
                .about("List most recently visited folders.")
                .arg(
                    Arg::with_name("number")
                        .empty_values(false)
                        .help("Jump to n-th most recently visited folder"),
                ),
        )
        .arg(
            Arg::with_name("name")
                .empty_values(true)
                .help("Refers to name of index. Must be specific for now."),
        )
}
