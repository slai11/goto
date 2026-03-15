use crate::db;
use anyhow::Result;
use itertools::{EitherOrBoth::*, Itertools};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

pub fn update(level: u8) -> Result<()> {
    let mut db = db::read_db()?;
    let path = env::current_dir()?;
    let mut cur = Vec::new();
    let mut next = Vec::new();
    cur.push(path);

    for _ in 0..=level {
        while let Some(p) = cur.pop() {
            // set immediate folder name as alias
            insert_path(&mut db, &p);

            // push sub-directories into next
            if p.is_dir() {
                for entry in fs::read_dir(p)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        next.push(path);
                    }
                }
            }
        }
        cur = next;
        next = Vec::new();
    }

    db::write_db(db)?;
    Ok(())
}

pub fn remove(level: u8) -> Result<()> {
    let mut db = db::read_db()?;
    let path = env::current_dir()?;
    let mut cur = Vec::new();
    let mut next = Vec::new();
    cur.push(path);

    let mut to_be_deleted: HashMap<String, bool> = HashMap::new();
    for _ in 0..=level {
        while let Some(p) = cur.pop() {
            // push sub-directories into next
            to_be_deleted.insert(p.display().to_string(), true);
            if p.is_dir() {
                for entry in fs::read_dir(p)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        next.push(path);
                    }
                }
            }
        }
        cur = next;
        next = Vec::new();
    }

    db.retain(|_, v| !to_be_deleted.contains_key(&v.path));

    db::write_db(db)?;
    Ok(())
}

pub fn prune() -> Result<()> {
    let pruned: HashMap<String, db::GotoFile> = db::read_db()?
        .into_iter()
        .filter(|(_, v)| Path::new(&v.path).exists())
        .collect();

    db::write_db(pruned)?;
    Ok(())
}

// insert_path handles the logic for inserting a new path.
// if another folder exists in the db with a different absolute path,
// `get_shortest_distinct_path` will generate the non-clashing alias pair.
pub fn insert_path(db: &mut HashMap<String, db::GotoFile>, p: &Path) {
    if db.values().any(|entry| entry.path == p.display().to_string()) {
        return;
    }

    let v = p.display().to_string();
    let k = p
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(&v)
        .to_string();

    match db.get(&k) {
        None => db.insert(
            k,
            db::GotoFile {
                path: v,
                count: 0,
                last_accessed: None,
            },
        ),
        Some(v_existing) => {
            if v_existing.path != p.display().to_string() {
                let (existing, clashing) = get_shortest_distinct_paths(&v_existing.path, &v);
                db.insert(
                    existing,
                    db::GotoFile {
                        path: v_existing.path.to_string(),
                        count: v_existing.count,
                        last_accessed: v_existing.last_accessed,
                    },
                );
                db.insert(
                    clashing,
                    db::GotoFile {
                        path: v,
                        count: 0,
                        last_accessed: None,
                    },
                );
                db.remove(&k);
            }
            None
        }
    };
}

fn get_shortest_distinct_paths(a: &str, b: &str) -> (String, String) {
    let asplit = a.split('/').collect::<Vec<&str>>();
    let bsplit = b.split('/').collect::<Vec<&str>>();
    let mut avec = Vec::new();
    let mut bvec = Vec::new();

    for pair in asplit.iter().rev().zip_longest(bsplit.iter().rev()) {
        match pair {
            Both(l, r) => {
                avec.push(*l);
                bvec.push(*r);
                if l != r {
                    break;
                }
            }
            Left(l) => {
                avec.push(l);
                break;
            }

            Right(r) => {
                bvec.push(r);
                break;
            }
        }
    }

    avec.reverse();
    bvec.reverse();
    (avec.join("/"), bvec.join("/"))
}

#[test]
fn test_diff_length() {
    let (a, b) = get_shortest_distinct_paths("a/b/c", "c");
    assert_eq!(a, "b/c");
    assert_eq!(b, "c");
}

#[test]
fn test_same_length() {
    let (a, b) = get_shortest_distinct_paths("a/b/c", "a/a/c");
    assert_eq!(a, "b/c");
    assert_eq!(b, "a/c");
}
