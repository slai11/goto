use std::collections::HashMap;
use std::io::{self, Write};

use crate::db;
use anyhow::{anyhow, Result};

// switch_to writes a single file path to stdout which will be
// picked up by the bash command to `cd` to.
pub fn switch_to(k: &str, exact: bool) -> Result<()> {
    let db = db::read_db()?;
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    let filepath = match exact {
        true => k.to_string(),
        false => match db.get(k) {
            Some(v) => v.path.clone(),
            None => fuzzy_lookup(&db, k).ok_or_else(|| {
                anyhow!(
                    "No such alias: {}, try using the `ls` command to list the aliases.",
                    k
                )
            })?,
        },
    };

    let _ = db::update_count(db, &filepath);
    writeln!(handle, "{}", filepath)?;
    Ok(())
}

// fuzzy_lookup filters aliases where the search term is a subsequence.
// Aliases are ranked according to how closely packed the subsequence is and ties
// are broken by length of the alias.
fn fuzzy_lookup(db: &HashMap<String, db::GotoFile>, w: &str) -> Option<String> {
    let candidates = db
        .iter()
        .map(|(alias, entry)| (alias.as_str(), entry, position_vec(alias, w)))
        .filter(|(_, _, positions)| positions.len() == w.len())
        .map(|(alias, entry, mut v)| {
            v = v.windows(2).map(|x| x[1] - x[0]).collect::<Vec<_>>();
            v.sort();
            (
                alias,
                entry,
                v.iter().copied().fold(0, |acc, x| acc * 10 + x),
            )
        })
        .collect::<Vec<_>>();

    candidates
        .iter()
        .min_by(|a, b| a.2.cmp(&b.2))
        .and_then(|(_, _, best_score)| {
            candidates
                .iter()
                .filter(|(_, _, score)| *score == *best_score)
                .min_by(|a, b| a.0.len().cmp(&b.0.len()))
                .map(|(_, entry, _)| entry.path.to_string())
        })
}

// position_vec returns a vec highlighting positions where search terms
// shows up in alias.
fn position_vec(alias: &str, path: &str) -> Vec<usize> {
    let mut alias_ptr = alias.chars();
    let mut vec = Vec::<usize>::new();
    let mut idx = 0;

    for c in path.chars() {
        // advance 1 char in alias until a match occurs
        loop {
            match alias_ptr.next() {
                None => return vec,
                Some(alias_c) => {
                    if alias_c == c {
                        vec.push(idx);
                        idx += 1;
                        break;
                    }
                }
            }
            idx += 1;
        }
    }

    vec
}

#[test]
fn fuzzy_test_takes_shortest() {
    let mut db = HashMap::new();
    db.insert(
        String::from("my-very-long-alias"),
        db::GotoFile {
            path: String::from("1"),
            count: 0,
        },
    );
    db.insert(
        String::from("my-very-xxx-alias"),
        db::GotoFile {
            path: String::from("2"),
            count: 0,
        },
    );
    assert_eq!(fuzzy_lookup(&db, "myalias").unwrap(), "2");
}

#[test]
fn fuzzy_test_takes_most_relevant() {
    let mut db = HashMap::new();
    db.insert(
        String::from("media_engine"),
        db::GotoFile {
            path: String::from("1"),
            count: 0,
        },
    );
    db.insert(
        String::from("manifest_services_so_long_name"),
        db::GotoFile {
            path: String::from("2"),
            count: 0,
        },
    );
    db.insert(
        String::from("man_paginator"),
        db::GotoFile {
            path: String::from("3"),
            count: 0,
        },
    );
    assert_eq!(fuzzy_lookup(&db, "mani").unwrap(), "2");
}
