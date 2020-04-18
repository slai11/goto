use std::io::{self, Write};

use crate::db;
use anyhow::{anyhow, Result};

// switch_to writes a single file path to stdout which will be
// picked up by the bash command to `cd` to.
pub fn switch_to(k: &str) -> Result<()> {
    let db = db::read_db()?;
    //TODO fuzzy search for best-fit
    match db.get(k) {
        Some(v) => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            writeln!(handle, "{}", &v).unwrap();
            Ok(())
        }

        None => Err(anyhow!(
            "No such alias: {}, try using the `ls` command to list the aliases."
        )),
    }
}
