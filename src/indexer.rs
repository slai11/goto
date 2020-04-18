use crate::db;
use anyhow::Result;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

pub fn update(level: i8) -> Result<()> {
    let mut db = db::read_db()?;
    let path = env::current_dir()?;
    let mut cur = Vec::new();
    let mut next = Vec::new();
    cur.push(path);

    for _ in 0..=level {
        while !cur.is_empty() {
            let p = cur.pop().unwrap();

            // set immediate folder name as alias
            db.insert(
                p.display()
                    .to_string()
                    .split("/")
                    .last()
                    .unwrap()
                    .to_string(),
                p.display().to_string(),
            );

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

pub fn prune() -> Result<()> {
    let pruned: HashMap<String, String> = db::read_db()?
        .into_iter()
        .filter(|(_, v)| Path::new(&v).exists())
        .collect();

    db::write_db(pruned)?;
    Ok(())
}
