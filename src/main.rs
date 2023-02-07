use anyhow::{anyhow, Result};
use std::process;

mod app;
mod db;
mod indexer;
mod init;
mod pretty_print;
mod switch;

fn run() -> Result<()> {
    let matches = app::build_app().get_matches();
    if let Some(matches) = matches.subcommand_matches("add") {
        let n: i8 = match matches.value_of("recursive") {
            Some(level) => level.parse().unwrap(),
            None => {
                if matches.is_present("all") {
                    1
                } else {
                    0
                }
            }
        };

        // perform add logic here
        return indexer::update(n);
    }

    if let Some(matches) = matches.subcommand_matches("rm") {
        let n: i8 = match matches.value_of("recursive") {
            Some(level) => level.parse().unwrap(),
            None => {
                if matches.is_present("all") {
                    1
                } else {
                    0
                }
            }
        };

        // perform add logic here
        return indexer::remove(n);
    }

    if matches.subcommand_matches("ls").is_some() {
        return db::list();
    }

    if matches.subcommand_matches("prune").is_some() {
        return indexer::prune();
    }

    if let Some(subcmd) = matches.subcommand_matches("jump") {
        return db::read_db()
            .map(|folders| {
                let jumpsites = db::list_jumpsites(folders);
                match subcmd.occurrences_of("number") {
                    1 => {
                        let idx = subcmd
                            .get_one::<String>("number")
                            .unwrap()
                            .trim()
                            .parse::<usize>()
                            .unwrap();
                        let site: &db::GotoFile = &jumpsites[idx - 1];
                        switch::switch_to(&site.path, true)
                    }
                    _ => pretty_print::pretty_print_jumpsites(&jumpsites),
                }
            })
            .and_then(|f| f);
    }

    if matches.subcommand_matches("init").is_some() {
        init::init();
        return Ok(());
    }

    match matches.occurrences_of("name") {
        1 => switch::switch_to(matches.value_of("name").unwrap(), false),
        _ => Err(anyhow!("Incorrect number of arguments.")),
    }
}

fn main() {
    let result = run();
    match result {
        Ok(_) => {
            process::exit(0);
        }
        Err(err) => {
            println!("[gt error]: {}", err);
            process::exit(1);
        }
    }
}
