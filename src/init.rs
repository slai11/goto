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
    cd "$@" || return "$?"
    if [ -n "$_ZO_ECHO" ]; then
        echo "$PWD"
    fi
}}
gt() {{
    if [[ -z $@ ]]; then
        goto-rs jump
        read jump_num
        result=$(goto-rs jump "$jump_num")
        cd $result
    else
        result="$(goto-rs "$@")" || return "$?"
        if [ -d "$result" ]; then
                _gt "$result" || return "$?"
            elif [ -n "$result" ]; then
                echo "$result"
        fi
    fi
}}
"#,
    )
}
