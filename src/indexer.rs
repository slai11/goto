use crate::db::{self, GotoFile};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn update(depth: u8) -> Result<()> {
    let mut db = db::read_db()?;
    for path in walk_current_dir(depth)? {
        insert_path(&mut db, &path);
    }
    db::write_db(&db)
}

pub fn remove(depth: u8) -> Result<()> {
    let mut db = db::read_db()?;
    let to_be_deleted = walk_current_dir(depth)?
        .into_iter()
        .map(|path| path.display().to_string())
        .collect::<HashSet<_>>();
    db.retain(|_, v| !to_be_deleted.contains(&v.path));
    db::write_db(&db)
}

pub fn prune() -> Result<()> {
    let mut db = db::read_db()?;
    db.retain(|_, v| Path::new(&v.path).exists());
    db::write_db(&db)
}

// insert_path handles the logic for inserting a new path.
// if another folder exists in the db with a different absolute path,
// `get_shortest_distinct_paths` will generate the non-clashing alias pair.
pub fn insert_path(db: &mut HashMap<String, GotoFile>, p: &Path) {
    let path = p.display().to_string();
    if db.values().any(|entry| entry.path == path) {
        return;
    }

    let alias = p
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(&path)
        .to_string();
    let entry = GotoFile {
        path,
        count: 0,
        last_accessed: None,
    };

    match db.remove(&alias) {
        None => {
            db.insert(alias, entry);
        }
        Some(existing) => {
            let (existing_alias, new_alias) =
                get_shortest_distinct_paths(&existing.path, &entry.path);
            db.insert(existing_alias, existing);
            db.insert(new_alias, entry);
        }
    }
}

// the current directory plus its non-hidden subdirectories, `depth` levels deep
fn walk_current_dir(depth: u8) -> Result<Vec<PathBuf>> {
    let mut found = vec![env::current_dir()?];
    let mut level_start = 0;
    for _ in 0..depth {
        let mut next = Vec::new();
        for dir in &found[level_start..] {
            collect_indexable_child_dirs(dir, &mut next)?;
        }
        level_start = found.len();
        found.append(&mut next);
    }
    Ok(found)
}

fn collect_indexable_child_dirs(dir: &Path, next: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() && !is_hidden_directory_name(&path) {
            next.push(path);
        }
    }
    Ok(())
}

fn is_hidden_directory_name(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.to_string_lossy().starts_with('.'))
}

// keeps the shared trailing components plus the first differing one, e.g.
// ("a/b/c", "a/a/c") -> ("b/c", "a/c")
fn get_shortest_distinct_paths(a: &str, b: &str) -> (String, String) {
    let a_parts = a.split('/').collect::<Vec<_>>();
    let b_parts = b.split('/').collect::<Vec<_>>();
    let shared = a_parts
        .iter()
        .rev()
        .zip(b_parts.iter().rev())
        .take_while(|(l, r)| l == r)
        .count();
    let suffix = |parts: &[&str]| parts[parts.len().saturating_sub(shared + 1)..].join("/");
    (suffix(&a_parts), suffix(&b_parts))
}

#[test]
fn test_diff_length() {
    let (a, b) = get_shortest_distinct_paths("a/b/c", "c");
    assert_eq!(a, "b/c");
    assert_eq!(b, "c");
}

#[test]
fn test_same_length() {
    let (a, b) = get_shortest_distinct_paths("a/b/c", "a/a/c");
    assert_eq!(a, "b/c");
    assert_eq!(b, "a/c");
}

#[test]
fn insert_path_rekeys_clashing_aliases() {
    let mut db = HashMap::new();
    insert_path(&mut db, Path::new("/a/x/notes"));
    db.get_mut("notes").unwrap().count = 3;
    insert_path(&mut db, Path::new("/b/x/notes"));

    let mut aliases = db.keys().map(String::as_str).collect::<Vec<_>>();
    aliases.sort_unstable();
    assert_eq!(aliases, ["a/x/notes", "b/x/notes"]);
    assert_eq!(db["a/x/notes"].count, 3);
    assert_eq!(db["b/x/notes"].path, "/b/x/notes");
}

#[test]
fn hidden_directory_names_start_with_dot() {
    assert!(is_hidden_directory_name(Path::new("/tmp/.pi")));
    assert!(is_hidden_directory_name(Path::new("/tmp/.claude")));
    assert!(is_hidden_directory_name(Path::new("/tmp/.git")));
    assert!(!is_hidden_directory_name(Path::new("/tmp/src")));
    assert!(!is_hidden_directory_name(Path::new("/tmp/foo.bar")));
}

#[test]
fn insert_path_keeps_explicit_hidden_directories() {
    let mut db = HashMap::new();
    insert_path(&mut db, Path::new("/tmp/.pi"));
    assert!(db.values().any(|entry| entry.path == "/tmp/.pi"));
}

#[cfg(unix)]
#[test]
fn hidden_directory_names_handle_non_utf8_names() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let name = OsString::from_vec(vec![b'.', b'p', b'i', 0xff]);
    assert!(is_hidden_directory_name(&Path::new("/tmp").join(name)));
}

#[test]
fn collect_indexable_child_dirs_skips_hidden_names() {
    let root = env::temp_dir().join(format!(
        "goto-hidden-index-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(root.join("visible")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join(".pi")).unwrap();
    fs::create_dir_all(root.join(".claude")).unwrap();
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join(".pi").join("skills")).unwrap();

    let mut next = Vec::new();
    collect_indexable_child_dirs(&root, &mut next).unwrap();
    let mut names: Vec<_> = next
        .iter()
        .filter_map(|path| path.file_name().and_then(|name| name.to_str()))
        .collect();
    names.sort_unstable();
    assert_eq!(names, ["src", "visible"]);

    let _ = fs::remove_dir_all(&root);
}
