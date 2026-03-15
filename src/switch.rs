use std::cmp::Reverse;
use std::collections::HashMap;
use std::io::{self, Write};

use crate::db;
use anyhow::{anyhow, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JumpOrder {
    Frecency,
    Recent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RankedPath {
    pub path: String,
    pub count: u32,
    pub last_accessed: Option<i64>,
}

pub fn switch_to_query(query: &[String]) -> Result<()> {
    let db = db::read_db()?;
    let now = db::now_ts()?;
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    let filepath = ranked_matches(&db, query, now, JumpOrder::Frecency)
        .into_iter()
        .next()
        .map(|entry| entry.path)
        .ok_or_else(|| {
            anyhow!(
                "No matching directory for query: {}. Try `gt search` or `gt ls`.",
                query.join(" ")
            )
        })?;

    let _ = db::touch_path(db, &filepath, now);
    writeln!(handle, "{}", filepath)?;
    Ok(())
}

pub fn switch_to_path(path: &str) -> Result<()> {
    let db = db::read_db()?;
    let now = db::now_ts()?;
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    let _ = db::touch_path(db, path, now);
    writeln!(handle, "{}", path)?;
    Ok(())
}

pub fn ranked_paths_for_jump(
    hm: HashMap<String, db::GotoFile>,
    now: i64,
    order: JumpOrder,
) -> Vec<RankedPath> {
    let mut ranked = hm
        .into_values()
        .filter(|entry| entry.count > 0)
        .map(|entry| RankedPath {
            path: entry.path,
            count: entry.count,
            last_accessed: entry.last_accessed,
        })
        .collect::<Vec<_>>();

    ranked.sort_by(|a, b| match order {
        JumpOrder::Frecency => (
            Reverse(frecency_score(a.count, a.last_accessed, now)),
            Reverse(a.last_accessed.unwrap_or_default()),
            a.path.as_str(),
        )
            .cmp(&(
                Reverse(frecency_score(b.count, b.last_accessed, now)),
                Reverse(b.last_accessed.unwrap_or_default()),
                b.path.as_str(),
            )),
        JumpOrder::Recent => (
            Reverse(a.last_accessed.unwrap_or_default()),
            Reverse(i64::from(a.count)),
            a.path.as_str(),
        )
            .cmp(&(
                Reverse(b.last_accessed.unwrap_or_default()),
                Reverse(i64::from(b.count)),
                b.path.as_str(),
            )),
    });
    ranked
}

pub fn ranked_matches(
    db: &HashMap<String, db::GotoFile>,
    query: &[String],
    now: i64,
    order: JumpOrder,
) -> Vec<RankedPath> {
    let terms = query
        .iter()
        .map(|term| term.trim())
        .filter(|term| !term.is_empty())
        .map(|term| term.to_ascii_lowercase())
        .collect::<Vec<_>>();

    let mut candidates = db
        .iter()
        .filter_map(|(alias, entry)| {
            let match_score = query_match_score(alias, &entry.path, &terms)?;
            let order_score = match order {
                JumpOrder::Frecency => frecency_score(entry.count, entry.last_accessed, now),
                JumpOrder::Recent => entry.last_accessed.unwrap_or_default(),
            };
            Some((
                Reverse(match_score),
                Reverse(order_score),
                Reverse(entry.last_accessed.unwrap_or_default()),
                Reverse(entry.count),
                alias.len(),
                entry.path.len(),
                RankedPath {
                    path: entry.path.clone(),
                    count: entry.count,
                    last_accessed: entry.last_accessed,
                },
            ))
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|a, b| {
        (&a.0, &a.1, &a.2, &a.3, &a.4, &a.5).cmp(&(&b.0, &b.1, &b.2, &b.3, &b.4, &b.5))
    });
    candidates
        .into_iter()
        .map(|(_, _, _, _, _, _, entry)| entry)
        .collect()
}

fn query_match_score(alias: &str, path: &str, terms: &[String]) -> Option<i64> {
    if terms.is_empty() {
        return Some(0);
    }

    let alias_lower = alias.to_ascii_lowercase();
    let path_lower = path.to_ascii_lowercase();
    let tokens = tokenize(alias).chain(tokenize(path)).collect::<Vec<_>>();

    let mut score = 0i64;
    for term in terms {
        let term_score = term_match_score(term, &alias_lower, &path_lower, &tokens)?;
        score += term_score;
    }

    Some(score)
}

fn term_match_score(
    term: &str,
    alias_lower: &str,
    path_lower: &str,
    tokens: &[String],
) -> Option<i64> {
    let mut best = 0i64;

    if alias_lower == term {
        best = best.max(900);
    }
    if path_lower.ends_with(&format!("/{}", term)) || path_lower == term {
        best = best.max(820);
    }

    for token in tokens {
        if token == term {
            best = best.max(700);
        } else if token.starts_with(term) {
            best = best.max(520 - (token.len() as i64 - term.len() as i64).min(40));
        } else if token.contains(term) {
            best = best.max(360 - (token.len() as i64 - term.len() as i64).min(40));
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

    if best > 0 {
        Some(best)
    } else {
        None
    }
}

fn tokenize(input: &str) -> impl Iterator<Item = String> + '_ {
    input
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_ascii_lowercase())
}

fn is_subsequence(haystack: &str, needle: &str) -> bool {
    let mut chars = haystack.chars();
    needle
        .chars()
        .all(|needle_char| chars.by_ref().any(|hay| hay == needle_char))
}

fn frecency_score(count: u32, last_accessed: Option<i64>, now: i64) -> i64 {
    let visit_score = i64::from(count).min(100) * 15;
    let recency_score = match last_accessed {
        Some(ts) => {
            let age = now.saturating_sub(ts);
            if age <= 60 * 60 {
                1200
            } else if age <= 24 * 60 * 60 {
                900
            } else if age <= 7 * 24 * 60 * 60 {
                600
            } else if age <= 30 * 24 * 60 * 60 {
                300
            } else {
                120
            }
        }
        None => 0,
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
fn frecency_beats_stale_high_count_entries() {
    let now = 10_000;
    let recent = frecency_score(4, Some(now - 60), now);
    let stale = frecency_score(40, Some(now - 120 * 24 * 60 * 60), now);
    assert!(recent > stale);
}
