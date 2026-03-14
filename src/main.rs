use anyhow::{anyhow, Result};
use itertools::Itertools;
use std::process;

use inquire::Select;

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
        Some(("add", sub_matches)) => {
            let depth = sub_matches
                .get_one::<u8>("recursive")
                .copied()
                .unwrap_or_else(|| if sub_matches.get_flag("all") { 1 } else { 0 });
            indexer::update(depth)
        }
        Some(("rm", sub_matches)) => {
            let depth = sub_matches
                .get_one::<u8>("recursive")
                .copied()
                .unwrap_or_else(|| if sub_matches.get_flag("all") { 1 } else { 0 });
            indexer::remove(depth)
        }
        Some(("jump", sub_matches)) => {
            let jumpsites = db::list_jumpsites(db::read_db()?);
            match sub_matches.get_one::<usize>("number").copied() {
                Some(0) => Err(anyhow!("Jump index must be at least 1.")),
                Some(index) => {
                    let site = jumpsites.get(index - 1).ok_or_else(|| {
                        anyhow!(
                            "Jump index {} is out of range. Available entries: {}.",
                            index,
                            jumpsites.len()
                        )
                    })?;
                    switch::switch_to(&site.path, true)
                }
                None => pretty_print::pretty_print_jumpsites(&jumpsites),
            }
        }

        Some(("search", _)) => {
            let jumpsites = db::read_db()?
                .values()
                .sorted_by(|a, b| Ord::cmp(&b.count, &a.count))
                .map(|g| g.path.clone())
                .collect::<Vec<_>>();

            if jumpsites.is_empty() {
                return Err(anyhow!(
                    "No jump history found yet. Use `gt <alias>` first or run `gt ls`."
                ));
            }

            let site = Select::new("Jump to:", jumpsites)
                .with_page_size(20)
                .prompt()?;

            switch::switch_to(&site, true)
        }

        _ => match matches.get_one::<String>("name") {
            Some(name) => switch::switch_to(name, false),
            _ => Err(anyhow!("Incorrect number of arguments.")),
        },
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("[gt error]: {}", err);
        process::exit(1);
    }
}
