use anyhow::Result;

const ZSH_INIT: &str = r#"
autoload -Uz add-zsh-hook

_gt_record() {
    command goto-rs record "${PWD:A}" >/dev/null 2>&1 || true
}

gt() {
    local result
    if [ "$#" -eq 0 ]; then
        result="$(command goto-rs search)" || return "$?"
    else
        result="$(command goto-rs "$@")" || return "$?"
    fi
    if [ -d "$result" ]; then
        builtin cd -- "$result"
    elif [ -n "$result" ]; then
        printf '%s\n' "$result"
    fi
}

add-zsh-hook chpwd _gt_record
_gt_record
"#;

pub fn init() -> Result<()> {
    println!("{}", ZSH_INIT);
    Ok(())
}
