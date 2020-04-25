use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::path::Path;

use anyhow::{anyhow, Result};
use csv::ReaderBuilder;
use csv::Writer;

use crate::pretty_print;

// reads k,v pairs from db, returning a hmap
pub fn read_db() -> Result<HashMap<String, String>> {
    let mut index_map = HashMap::new();
    match env::home_dir().map(|p| format!("{}/{}", p.display(), ".config/goto/db.txt")) {
        Some(path) => {
            if !Path::new(&path).exists() {
                init_db(path.to_string());
            }
            let file = File::open(path)?;
            let mut rdr = ReaderBuilder::new().has_headers(false).from_reader(file);

            for result in rdr.records() {
                let record = result?;
                let alias = &record[0];
                let fp = &record[1];
                index_map.insert(String::from(alias), String::from(fp));
            }

            Ok(index_map)
        }

        None => Err(anyhow!("No home directory available!")),
    }
}

pub fn write_db(hm: HashMap<String, String>) -> Result<()> {
    match env::home_dir().map(|p| format!("{}/{}", p.display(), ".config/goto/db.txt")) {
        Some(path) => {
            let mut wtr = Writer::from_path(path)?;
            for (k, v) in &hm {
                wtr.write_record(&[k, v])?;
            }
            wtr.flush()?;
            Ok(())
        }
        None => Err(anyhow!("gt's index database does not exist!")),
    }
}

fn init_db(path: String) -> Result<()> {
    File::create(path)?;
    Ok(())
}

pub fn list() -> Result<()> {
    let db = read_db()?;
    let mut v = db.iter().collect::<Vec<_>>();

    println!("======= Current Indexed Directories (alias highlighted) =======");
    v.sort_by(|a, b| a.1.cmp(&b.1));
    pretty_print::pretty_print(&v)
}
