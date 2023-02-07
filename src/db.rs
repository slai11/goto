use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::Path;

use anyhow::{anyhow, Result};
use csv::ReaderBuilder;
use csv::Writer;
use dirs;

use crate::pretty_print;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GotoFile {
    pub count: u32,
    pub path: String,
}

// reads k,v pairs from db, returning a hmap
pub fn read_db() -> Result<HashMap<String, GotoFile>> {
    let mut index_map = HashMap::new();
    let filename = "db.txt";
    match dirs::home_dir().map(|p| format!("{}/{}", p.display(), ".config/goto")) {
        Some(path) => {
            let filepath = format!("{}/{}", path, filename);
            if !Path::new(&filepath).exists() {
                init_db(path.to_string(), filename.to_string())?;
            }
            let file = File::open(filepath)?;
            let mut rdr = ReaderBuilder::new().has_headers(false).from_reader(file);

            for result in rdr.records() {
                let record = result?;
                let alias = &record[0];
                let fp = &record[1];
                let freq_count: u32 = if record.len() > 2 {
                    record[2].parse::<u32>().unwrap_or(0)
                } else {
                    0
                };
                index_map.insert(
                    String::from(alias),
                    GotoFile {
                        path: String::from(fp),
                        count: freq_count,
                    },
                );
            }

            Ok(index_map)
        }

        None => Err(anyhow!("No home directory available!")),
    }
}

pub fn write_db(hm: HashMap<String, GotoFile>) -> Result<()> {
    match dirs::home_dir().map(|p| format!("{}/{}", p.display(), ".config/goto/db.txt")) {
        Some(path) => {
            let mut wtr = Writer::from_path(path)?;
            for (k, v) in &hm {
                wtr.write_record(&[k, &v.path, &v.count.to_string()])?;
            }
            wtr.flush()?;
            Ok(())
        }
        None => Err(anyhow!("gt's index database does not exist!")),
    }
}

pub fn update_count(mut hm: HashMap<String, GotoFile>, key: &String) {
    for (_, val) in hm.iter_mut() {
        if val.path == *key {
            val.count += 1;
            write_db(hm); // ignore errors, we can skip updates
            break;
        }
    }
}

fn init_db(path: String, filename: String) -> Result<()> {
    fs::create_dir_all(&path)?;
    let filepath = format!("{}/{}", path, filename);
    File::create(filepath)?;
    Ok(())
}

pub fn list() -> Result<()> {
    let db = read_db()?;
    let mut v = db.iter().collect::<Vec<_>>();

    println!("======= Current Indexed Directories (alias highlighted) =======");
    v.sort_by(|a, b| a.1.path.cmp(&b.1.path));
    pretty_print::pretty_print_tree(&v)
}

pub fn list_jumpsites(hm: HashMap<String, GotoFile>) -> Vec<GotoFile> {
    let mut vec: Vec<GotoFile> = Vec::new();
    for (_, val) in hm.iter() {
        if val.count > 0 {
            vec.push(val.clone());
        }
    }
    vec.sort();
    vec.reverse();
    vec
}
