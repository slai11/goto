use anyhow::Result;
use std::io::{self, Write};

pub fn init() -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    writeln!(handle, "{}", posix_goto())?;
    Ok(())
}

fn posix_goto() -> String {
    String::from(
        r#"
_gt() {{
    cd -- "$@" || return "$?"
    if [ -n "$_ZO_ECHO" ]; then
        printf '%s\n' "$PWD"
    fi
}}
gt() {{
    if [ "$#" -eq 0 ]; then
        result="$(goto-rs search)" || return "$?"
        if [ -n "$result" ]; then
            cd -- "$result" || return "$?"
        fi
    else
        result="$(goto-rs "$@")" || return "$?"
        if [ -d "$result" ]; then
            _gt "$result" || return "$?"
        elif [ -n "$result" ]; then
            printf '%s\n' "$result"
        fi
    fi
}}
"#,
    )
}
