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
    match matches.subcommand() {
        Some(("ls", _)) => db::list(),
        Some(("prune", _)) => indexer::prune(),
        Some(("init", _)) => init::init(),
        Some(("add", sub_matches)) | Some(("rm", sub_matches)) => {
            let n: u8 = match sub_matches.get_one::<String>("recursive") {
                Some(level) => level.trim().parse::<u8>().unwrap(),
                None => match sub_matches.get_one::<bool>("all") {
                    Some(true) => 1,
                    _ => 0,
                },
            };

            // perform add logic here
            match matches.subcommand() {
                Some(("add", _)) => indexer::update(n),
                Some(("rm", _)) => indexer::remove(n),
                _ => Ok(()),
            }
        }
        Some(("jump", sub_matches)) => db::read_db()
            .map(|folders| {
                let jumpsites = db::list_jumpsites(folders);
                match sub_matches.get_one::<String>("number") {
                    Some(idx_str) => {
                        let idx = idx_str.trim().parse::<usize>().unwrap();
                        let site: &db::GotoFile = &jumpsites[idx - 1];
                        switch::switch_to(&site.path, true)
                    }
                    _ => pretty_print::pretty_print_jumpsites(&jumpsites),
                }
            })
            .and_then(|f| f),

        _ => match matches.get_one::<String>("name") {
            Some(name) => switch::switch_to(name, false),
            _ => Err(anyhow!("Incorrect number of arguments.")),
        },
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
