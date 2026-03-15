use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use csv::ReaderBuilder;
use csv::Writer;

use crate::indexer;
use crate::pretty_print;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GotoFile {
    pub count: u32,
    pub last_accessed: Option<i64>,
    pub path: String,
}

// reads k,v pairs from db, returning a hmap
pub fn read_db() -> Result<HashMap<String, GotoFile>> {
    let mut index_map = HashMap::new();
    let path = db_path()?;
    if !Path::new(&path).exists() {
        init_db(&path)?;
    }
    let file = File::open(path)?;
    let mut rdr = ReaderBuilder::new().has_headers(false).from_reader(file);

    for result in rdr.records() {
        let record = result?;
        let alias = record
            .get(0)
            .ok_or_else(|| anyhow!("Malformed database row: missing alias."))?;
        let file_path = record
            .get(1)
            .ok_or_else(|| anyhow!("Malformed database row: missing path."))?;
        let freq_count = record
            .get(2)
            .and_then(|count| count.parse::<u32>().ok())
            .unwrap_or(0);
        let last_accessed = record.get(3).and_then(|ts| ts.parse::<i64>().ok());

        index_map.insert(
            alias.to_string(),
            GotoFile {
                path: file_path.to_string(),
                count: freq_count,
                last_accessed,
            },
        );
    }

    Ok(index_map)
}

pub fn write_db(hm: HashMap<String, GotoFile>) -> Result<()> {
    let path = db_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut wtr = Writer::from_path(path)?;
    for (k, v) in &hm {
        let count = v.count.to_string();
        let last_accessed = v.last_accessed.map(|ts| ts.to_string()).unwrap_or_default();
        wtr.write_record([
            k.as_str(),
            v.path.as_str(),
            count.as_str(),
            last_accessed.as_str(),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn touch_path(mut hm: HashMap<String, GotoFile>, key: &str, now: i64) -> Result<()> {
    let mut updated = false;
    for val in hm.values_mut() {
        if val.path == key {
            val.count += 1;
            val.last_accessed = Some(now);
            updated = true;
            break;
        }
    }
    if updated {
        write_db(hm)?;
    }
    Ok(())
}

pub fn learn_path(path: &Path, now: i64) -> Result<()> {
    let mut db = read_db()?;
    indexer::insert_path(&mut db, path);
    touch_path(db, &path.display().to_string(), now)
}

pub fn now_ts() -> Result<i64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow!("System clock is before UNIX epoch."))?
        .as_secs() as i64)
}

fn db_path() -> Result<PathBuf> {
    let mut path = dirs::home_dir().ok_or_else(|| anyhow!("No home directory available!"))?;
    path.push(".config");
    path.push("goto");
    path.push("db.txt");
    Ok(path)
}

fn init_db(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    File::create(path)?;
    Ok(())
}

pub fn list() -> Result<()> {
    let db = read_db()?;
    let mut v = db.iter().collect::<Vec<_>>();

    println!("======= Current Indexed Directories (alias highlighted) =======");
    v.sort_by(|a, b| a.1.path.cmp(&b.1.path));
    pretty_print::pretty_print_tree(&v)
}
