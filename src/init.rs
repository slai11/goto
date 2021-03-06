use std::io::{self, Write};

pub fn init() {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    writeln!(handle, "{}", posix_goto()).unwrap();
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
    result="$(goto-rs "$@")" || return "$?"
    if [ -d "$result" ]; then
            _gt "$result" || return "$?"
        elif [ -n "$result" ]; then
            echo "$result"
    fi
}}
"#,
    )
}
