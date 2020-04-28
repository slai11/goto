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
        .filter(|(k, _)| in_alias(&k, w))
        .collect::<Vec<_>>();

    // NOTE: take shortest alias (assumes to be closest match)
    v.sort_by(|a, b| a.0.len().cmp(&b.0.len()));
    v.first().map(|(_, v)| v.to_string())
}

// checks if user-entered path is a subsequence of alias
// Time-Complexity O(m+n)
fn in_alias(alias: &str, path: &str) -> bool {
    let mut alias_ptr = alias.chars();

    for c in path.chars() {
        // advance 1 char in alias until a match occurs
        while let opt_char = alias_ptr.nth(0) {
            match opt_char {
                None => return false,
                Some(alias_c) => {
                    if alias_c == c {
                        // advance to next char in word on a match
                        break;
                    }
                }
            }
        }
    }

    true
}

#[test]
fn alias_test() {
    assert_eq!(in_alias("alias", "ais"), true);
    assert_eq!(in_alias("alias", "ila"), false);

    // empty alias
    assert_eq!(in_alias("", "abc"), false);

    // path longer than alias
    assert_eq!(in_alias("a", "abc"), false);
}

#[test]
fn fuzzy_test_takes_shortest() {
    let mut db = HashMap::new();
    db.insert(String::from("my-very-long-alias"), String::from("1"));
    db.insert(String::from("my-very-xxx-alias"), String::from("2"));
    assert_eq!(fuzzy_lookup(db, "myalias").unwrap(), "2");
}
