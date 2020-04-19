use anyhow::{anyhow, Result};
use std::process;

mod app;
mod db;
mod indexer;
mod init;
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

    if let Some(_) = matches.subcommand_matches("ls") {
        return db::list();
    }

    if let Some(_) = matches.subcommand_matches("prune") {
        return indexer::prune();
    }

    if let Some(_) = matches.subcommand_matches("init") {
        init::init();
        return Ok(());
    }

    match matches.occurrences_of("name") {
        1 => switch::switch_to(matches.value_of("name").unwrap()),
        _ => Err(anyhow!("No home directory available!")),
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
