use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::path::Path;

use anyhow::{anyhow, Result};
use csv::ReaderBuilder;
use csv::Writer;

// reads k,v pairs from db, returning a hmap
pub fn read_db() -> Result<HashMap<String, String>> {
    let mut index_map = HashMap::new();
    let db_path = env::home_dir().map(|p| format!("{}/{}", p.display(), ".config/goto/db.txt"));

    match db_path {
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
    let db_path = "/Users/sylvesterchin/.config/goto/db.txt";
    let mut wtr = Writer::from_path(db_path)?;
    for (k, v) in &hm {
        wtr.write_record(&[k, v])?;
    }
    wtr.flush()?;
    Ok(())
}

fn init_db(path: String) -> Result<()> {
    File::create(path)?;
    Ok(())
}
