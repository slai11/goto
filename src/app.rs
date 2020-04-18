use clap::{App, AppSettings, Arg, SubCommand};

pub fn build_app() -> App<'static, 'static> {
    let app = App::new("gt")
        .setting(AppSettings::ColoredHelp)
        .subcommand(
            SubCommand::with_name("init").about("Add directories and sub-directories to index"),
        )
        .subcommand(
            SubCommand::with_name("prune").about("Add directories and sub-directories to index"),
        )
        .subcommand(
            SubCommand::with_name("add")
                .about("Add directories and sub-directories to index")
                .arg(
                    Arg::with_name("recursive")
                        .short("r")
                        .takes_value(true)
                        .help("Recursively indexs"),
                ),
        )
        .arg(
            Arg::with_name("name")
                .multiple(true)
                .empty_values(true)
                .help("Refers to name of index. Must be specific for now"),
        );
    app
}
