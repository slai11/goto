use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::{Component, Path};

use crate::db::{self, GotoFile};
use anyhow::{Result, anyhow};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JumpOrder {
    Frecency,
    Recent,
}

pub fn switch_to_query(query: &[String]) -> Result<()> {
    let db = db::read_db()?;
    let path = ranked_matches(&db, query, db::now_ts()?, JumpOrder::Frecency)
        .into_iter()
        .next()
        .map(|entry| entry.path)
        .ok_or_else(|| {
            anyhow!(
                "No matching directory for query: {}. Try `gt search` or `gt ls`.",
                query.join(" ")
            )
        })?;
    switch_to_path(&path)
}

// prints `path` for the shell wrapper to cd into, recording the visit on a best-effort basis
pub fn switch_to_path(path: &str) -> Result<()> {
    let _ = db::touch_path(db::read_db()?, path, db::now_ts()?);
    println!("{}", path);
    Ok(())
}

// visited directories only, best first
pub fn ranked_paths_for_jump(
    db: &HashMap<String, GotoFile>,
    now: i64,
    order: JumpOrder,
) -> Vec<GotoFile> {
    ranked_matches(db, &[], now, order)
        .into_iter()
        .filter(|entry| entry.count > 0)
        .collect()
}

// entries matching every query term, sorted by match quality and then by `order`.
// paths through a hidden directory (`.pi/worktrees`, `.claude/worktrees`, …) are omitted.
pub fn ranked_matches(
    db: &HashMap<String, GotoFile>,
    query: &[String],
    now: i64,
    order: JumpOrder,
) -> Vec<GotoFile> {
    let terms = query
        .iter()
        .map(|term| term.trim().to_ascii_lowercase())
        .filter(|term| !term.is_empty())
        .collect::<Vec<_>>();

    let mut candidates = db
        .iter()
        .filter(|(_, entry)| !path_contains_hidden_dir(&entry.path))
        .filter_map(|(alias, entry)| {
            let match_score = query_match_score(alias, &entry.path, &terms)?;
            let last_accessed = entry.last_accessed.unwrap_or_default();
            let order_score = match order {
                JumpOrder::Frecency => frecency_score(entry.count, entry.last_accessed, now),
                JumpOrder::Recent => last_accessed,
            };
            let key = (
                Reverse(match_score),
                Reverse(order_score),
                Reverse(last_accessed),
                Reverse(entry.count),
                entry.path.len(),
                entry.path.as_str(),
            );
            Some((key, entry))
        })
        .collect::<Vec<_>>();

    candidates.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    candidates
        .into_iter()
        .map(|(_, entry)| entry.clone())
        .collect()
}

// sum of per-term scores; None if any term fails to match
fn query_match_score(alias: &str, path: &str, terms: &[String]) -> Option<i64> {
    let alias_lower = alias.to_ascii_lowercase();
    let path_lower = path.to_ascii_lowercase();
    let tokens = tokenize(&alias_lower)
        .chain(tokenize(&path_lower))
        .collect::<Vec<_>>();

    terms
        .iter()
        .map(|term| term_match_score(term, &alias_lower, &path_lower, &tokens))
        .sum()
}

fn term_match_score(
    term: &str,
    alias_lower: &str,
    path_lower: &str,
    tokens: &[&str],
) -> Option<i64> {
    let mut best = 0i64;

    if alias_lower == term {
        best = best.max(900);
    }
    if path_lower == term || path_lower.ends_with(&format!("/{}", term)) {
        best = best.max(820);
    }

    for token in tokens {
        // longer tokens score slightly lower, capped at a 40-point penalty
        let slack = token.len().saturating_sub(term.len()).min(40) as i64;
        if *token == term {
            best = best.max(700);
        } else if token.starts_with(term) {
            best = best.max(520 - slack);
        } else if token.contains(term) {
            best = best.max(360 - slack);
        }
    }

    if alias_lower.contains(term) {
        best = best.max(320);
    }
    if path_lower.contains(term) {
        best = best.max(280);
    }
    if is_subsequence(alias_lower, term) {
        best = best.max(200);
    }
    if is_subsequence(path_lower, term) {
        best = best.max(170);
    }

    (best > 0).then_some(best)
}

fn path_contains_hidden_dir(path: &str) -> bool {
    Path::new(path)
        .components()
        .any(|component| match component {
            Component::Normal(name) => name.to_string_lossy().starts_with('.'),
            _ => false,
        })
}

fn tokenize(input: &str) -> impl Iterator<Item = &str> {
    input
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
}

fn is_subsequence(haystack: &str, needle: &str) -> bool {
    let mut chars = haystack.chars();
    needle
        .chars()
        .all(|needle_char| chars.by_ref().any(|hay| hay == needle_char))
}

fn frecency_score(count: u32, last_accessed: Option<i64>, now: i64) -> i64 {
    const HOUR: i64 = 60 * 60;
    const DAY: i64 = 24 * HOUR;

    let visit_score = i64::from(count.min(100)) * 15;
    let recency_score = match last_accessed.map(|ts| now.saturating_sub(ts)) {
        None => 0,
        Some(age) if age <= HOUR => 1200,
        Some(age) if age <= DAY => 900,
        Some(age) if age <= 7 * DAY => 600,
        Some(age) if age <= 30 * DAY => 300,
        Some(_) => 120,
    };

    visit_score + recency_score
}

#[test]
fn ranked_matches_use_path_terms_and_multiple_words() {
    let mut db = HashMap::new();
    db.insert(
        String::from("personal"),
        db::GotoFile {
            path: String::from("/Users/sylvester/work/client-alpha/personal"),
            count: 3,
            last_accessed: Some(1_000),
        },
    );
    db.insert(
        String::from("client-beta/personal"),
        db::GotoFile {
            path: String::from("/Users/sylvester/work/client-beta/personal"),
            count: 5,
            last_accessed: Some(2_000),
        },
    );
    let ranked = ranked_matches(
        &db,
        &[String::from("client"), String::from("beta")],
        3_000,
        JumpOrder::Frecency,
    );
    assert_eq!(
        ranked.first().map(|entry| entry.path.as_str()),
        Some("/Users/sylvester/work/client-beta/personal")
    );
}

#[test]
fn ranked_matches_prefer_recent_when_match_quality_ties() {
    let mut db = HashMap::new();
    db.insert(
        String::from("notes"),
        db::GotoFile {
            path: String::from("/tmp/notes"),
            count: 1,
            last_accessed: Some(2_000),
        },
    );
    db.insert(
        String::from("archive/notes"),
        db::GotoFile {
            path: String::from("/tmp/archive/notes"),
            count: 20,
            last_accessed: Some(100),
        },
    );
    let ranked = ranked_matches(&db, &[String::from("notes")], 2_100, JumpOrder::Frecency);
    assert_eq!(
        ranked.first().map(|entry| entry.path.as_str()),
        Some("/tmp/notes")
    );
}

#[test]
fn ranked_matches_skip_paths_under_hidden_directories() {
    let mut db = HashMap::new();
    db.insert(
        String::from("pymporal_service"),
        db::GotoFile {
            path: String::from(
                "/Users/sylvester/allium/10tbps-asa/allium-services/pymporal_service",
            ),
            count: 2,
            last_accessed: Some(1_000),
        },
    );
    db.insert(
        String::from(".pi/worktrees/pymporal_service"),
        db::GotoFile {
            path: String::from(
                "/Users/sylvester/allium/10tbps/.pi/worktrees/pi-wt/allium-services/pymporal_service",
            ),
            count: 50,
            last_accessed: Some(9_000),
        },
    );
    db.insert(
        String::from(".claude/worktrees/pymporal_service"),
        db::GotoFile {
            path: String::from(
                "/Users/sylvester/allium/10tbps/.claude/worktrees/crystal/allium-services/pymporal_service",
            ),
            count: 50,
            last_accessed: Some(9_000),
        },
    );
    db.insert(
        String::from(".hidden"),
        db::GotoFile {
            path: String::from("/tmp/.hidden"),
            count: 10,
            last_accessed: Some(9_000),
        },
    );
    db.insert(
        String::from("foo.bar"),
        db::GotoFile {
            path: String::from("/tmp/foo.bar"),
            count: 1,
            last_accessed: Some(500),
        },
    );

    let ranked = ranked_matches(
        &db,
        &[String::from("pymporal")],
        10_000,
        JumpOrder::Frecency,
    );
    assert_eq!(
        ranked
            .iter()
            .map(|entry| entry.path.as_str())
            .collect::<Vec<_>>(),
        ["/Users/sylvester/allium/10tbps-asa/allium-services/pymporal_service"]
    );

    let visible = ranked_matches(&db, &[], 10_000, JumpOrder::Frecency);
    let paths = visible
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        [
            "/Users/sylvester/allium/10tbps-asa/allium-services/pymporal_service",
            "/tmp/foo.bar",
        ]
    );
}

#[test]
fn frecency_beats_stale_high_count_entries() {
    let now = 10_000;
    let recent = frecency_score(4, Some(now - 60), now);
    let stale = frecency_score(40, Some(now - 120 * 24 * 60 * 60), now);
    assert!(recent > stale);
}
