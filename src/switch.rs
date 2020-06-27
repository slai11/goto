use std::collections::HashMap;
use std::io::{self, Write};

use crate::db;
use anyhow::{anyhow, Result};

// switch_to writes a single file path to stdout which will be
// picked up by the bash command to `cd` to.
pub fn switch_to(k: &str) -> Result<()> {
    let db = db::read_db()?;
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    match db.get(k) {
        Some(v) => {
            writeln!(handle, "{}", &v).unwrap();
            Ok(())
        }

        None => match fuzzy_lookup(db, k) {
            None => Err(anyhow!(format!(
                "No such alias: {}, try using the `ls` command to list the aliases.",
                k
            ))),
            Some(fk) => {
                writeln!(handle, "{}", &fk).unwrap();
                Ok(())
            }
        },
    }
}

// fuzzy_lookup is not a try fuzzy lookup but merely a simple subsequence search
fn fuzzy_lookup(db: HashMap<String, String>, w: &str) -> Option<String> {
    let mut v = db
        .iter()
        .map(|(k, v)| (v, position_vec(&k, w)))
        .filter(|(_, v)| v.len() == w.len())
        .map(|(k, mut v)| {
            v = v.windows(2).map(|x| x[1] - x[0]).collect::<Vec<_>>();
            v.sort();
            (k, v.iter().fold(0, |acc, x| acc * 10 + x))
        })
        .collect::<Vec<_>>();

    // NOTE: take shortest alias (assumes to be closest match)
    v.sort_by(|a, b| a.1.cmp(&b.1));
    v.first().map(|(k, _)| k.to_string())
}

fn position_vec(alias: &str, path: &str) -> Vec<i32> {
    let mut alias_ptr = alias.chars();
    let mut vec = Vec::<i32>::new();
    let mut idx = 0;

    for c in path.chars() {
        // advance 1 char in alias until a match occurs
        while let opt_char = alias_ptr.nth(0) {
            match opt_char {
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
    db.insert(String::from("my-very-long-alias"), String::from("1"));
    db.insert(String::from("my-very-xxx-alias"), String::from("2"));
    assert_eq!(fuzzy_lookup(db, "myalias").unwrap(), "2");
}

#[test]
fn fuzzy_test_takes_most_relevant() {
    let mut db = HashMap::new();
    db.insert(String::from("media_engine"), String::from("1"));
    db.insert(
        String::from("manifest_services_so_long_name"),
        String::from("2"),
    );
    db.insert(String::from("man_paginator"), String::from("3"));
    assert_eq!(fuzzy_lookup(db, "mani").unwrap(), "2");
}
