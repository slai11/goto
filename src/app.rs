use clap::{arg, Command};

pub fn build_app() -> Command {
    Command::new("gt")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(false)
        .subcommand(Command::new("init").about("Initialises bash-script and database."))
        .subcommand(Command::new("ls").about("List all indexed directories."))
        .subcommand(Command::new("prune").about("Removes invalid indexes in the database."))
        .subcommand(
            Command::new("add")
                .about("Add directories and sub-directories to index.")
                .arg(arg!(-a --all "Adds all subdirectory."))
                .arg(arg!(-r --recursive "Recursively indexs.")),
        )
        .subcommand(
            Command::new("rm")
                .about("Remove directories and sub-directories to index.")
                .arg(arg!(-a --all "Removes all subdirectory."))
                .arg(arg!(-r --recursive "Recursively removes.")),
        )
        .subcommand(
            Command::new("jump")
                .about("List most recently visited folders.")
                .arg(arg!([number] "Jump to n-th most recently visited folder")),
        )
        .arg(arg!([name] "Refers to name of index. Must be specific for now."))
}
