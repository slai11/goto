use clap::{arg, value_parser, ArgAction, Command};

pub fn build_app() -> Command {
    Command::new("gt")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(false)
        .arg_required_else_help(true)
        .subcommand(Command::new("init").about("Initialises bash-script and database."))
        .subcommand(Command::new("ls").about("List all indexed directories."))
        .subcommand(Command::new("prune").about("Removes invalid indexes in the database."))
        .subcommand(
            Command::new("add")
                .about("Add directories and sub-directories to index.")
                .arg(arg!(-a --all "Adds one level of subdirectories.").action(ArgAction::SetTrue))
                .arg(
                    arg!(-r --recursive <depth> "Recursively indexes up to the provided depth.")
                        .value_parser(value_parser!(u8)),
                ),
        )
        .subcommand(
            Command::new("rm")
                .about("Remove directories and sub-directories to index.")
                .arg(
                    arg!(-a --all "Removes one level of subdirectories.")
                        .action(ArgAction::SetTrue),
                )
                .arg(
                    arg!(-r --recursive <depth> "Recursively removes up to the provided depth.")
                        .value_parser(value_parser!(u8)),
                ),
        )
        .subcommand(
            Command::new("jump")
                .about("List most recently visited folders.")
                .arg(
                    arg!([number] "Jump to the numbered most recently visited folder.")
                        .value_parser(value_parser!(usize)),
                ),
        )
        .subcommand(Command::new("search").about("Launches interactive select list."))
        .arg(arg!([name] "Refers to name of index. Must be specific for now."))
}
