use anyhow::{Result, anyhow};
use clap::ArgMatches;
use inquire::Select;
use std::path::Path;
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
        Some(("record", sub_matches)) => {
            let path = sub_matches
                .get_one::<String>("path")
                .ok_or_else(|| anyhow!("Path is required."))?;
            db::learn_path(Path::new(path), db::now_ts()?)
        }
        Some(("add", sub_matches)) => indexer::update(depth(sub_matches)),
        Some(("rm", sub_matches)) => indexer::remove(depth(sub_matches)),
        Some(("jump", sub_matches)) => {
            let order = if sub_matches.get_flag("recent") {
                switch::JumpOrder::Recent
            } else {
                switch::JumpOrder::Frecency
            };
            let jumpsites = switch::ranked_paths_for_jump(&db::read_db()?, db::now_ts()?, order);
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
                    switch::switch_to_path(&site.path)
                }
                None => pretty_print::pretty_print_jumpsites(&jumpsites),
            }
        }
        Some(("search", sub_matches)) => {
            let jumpsites = switch::ranked_matches(
                &db::read_db()?,
                &query_terms(sub_matches),
                db::now_ts()?,
                switch::JumpOrder::Frecency,
            )
            .into_iter()
            .map(|entry| entry.path)
            .collect::<Vec<_>>();

            if jumpsites.is_empty() {
                return Err(anyhow!(
                    "No learned directories found yet. Use `gt` normally or `gt add` first."
                ));
            }

            let site = Select::new("Jump to:", jumpsites)
                .with_page_size(20)
                .prompt()?;
            switch::switch_to_path(&site)
        }
        _ => {
            let query = query_terms(&matches);
            if query.is_empty() {
                Err(anyhow!("Provide a query or use `gt search`."))
            } else {
                switch::switch_to_query(&query)
            }
        }
    }
}

// `-r <depth>` wins over `-a`, which is shorthand for one level of subdirectories.
fn depth(matches: &ArgMatches) -> u8 {
    matches
        .get_one::<u8>("recursive")
        .copied()
        .unwrap_or(u8::from(matches.get_flag("all")))
}

fn query_terms(matches: &ArgMatches) -> Vec<String> {
    matches
        .get_many::<String>("query")
        .map(|values| values.cloned().collect())
        .unwrap_or_default()
}

fn main() {
    if let Err(err) = run() {
        eprintln!("[gt error]: {}", err);
        process::exit(1);
    }
}
