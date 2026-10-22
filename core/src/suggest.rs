//! Подсказки адресной строки по закладкам и истории.
//!
//! Вход — строки `вид \t заголовок \t url \t посещений \t последнее_посещение_unix_сек`,
//! где вид: `b` (закладка) или `h` (история). Выход — JSON-массив
//! `[{"k":"b","title":"…","url":"…"},…]`, лучшие совпадения первыми.

use crate::json;
use crate::url_util;
use std::collections::HashMap;

struct Entry<'a> {
    kind: &'a str,
    title: &'a str,
    url: &'a str,
    visits: f64,
    last: i64,
}

fn strip_scheme(url: &str) -> &str {
    let u = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    u.strip_prefix("www.").unwrap_or(u)
}

fn dedup_key(url: &str) -> String {
    strip_scheme(url).trim_end_matches('/').to_lowercase()
}

fn score(e: &Entry, tokens: &[String], now: i64) -> Option<f64> {
    let title = e.title.to_lowercase();
    let url = e.url.to_lowercase();
    let bare = strip_scheme(&url).to_string();
    let host = url_util::host_of(&url);
    let host = host.strip_prefix("www.").unwrap_or(&host).to_string();

    let mut s = 0.0;
    for t in tokens {
        let in_title = title.contains(t.as_str());
        let in_url = url.contains(t.as_str());
        if !in_title && !in_url {
            return None;
        }
        if bare.starts_with(t.as_str()) {
            s += 60.0;
        } else if host.starts_with(t.as_str()) {
            s += 50.0;
        } else if host.contains(t.as_str()) {
            s += 25.0;
        } else if in_url {
            s += 6.0;
        }
        if title.starts_with(t.as_str()) {
            s += 30.0;
        } else if title.split(|c: char| !c.is_alphanumeric()).any(|w| w.starts_with(t.as_str())) {
            s += 18.0;
        } else if in_title {
            s += 8.0;
        }
    }

    if e.kind == "b" {
        s += 15.0;
    }
    s += (1.0 + e.visits.max(0.0)).ln() * 5.0;
    if e.last > 0 && now >= e.last {
        let age_days = (now - e.last) as f64 / 86_400.0;
        s += 10.0 / (1.0 + age_days);
    }
    Some(s)
}

pub fn suggest(query: &str, entries: &str, limit: usize, now: i64) -> String {
    let tokens: Vec<String> = query
        .split_whitespace()
        .map(|t| t.to_lowercase())
        .filter(|t| !t.is_empty())
        .collect();
    if tokens.is_empty() || limit == 0 {
        return "[]".to_string();
    }

    let mut best: HashMap<String, (f64, Entry)> = HashMap::new();
    for line in entries.lines() {
        let mut f = line.split('\t');
        let (Some(kind), Some(title), Some(url)) = (f.next(), f.next(), f.next()) else {
            continue;
        };
        if url.is_empty() {
            continue;
        }
        let visits = f.next().and_then(|v| v.trim().parse::<f64>().ok()).unwrap_or(0.0);
        let last = f.next().and_then(|v| v.trim().parse::<i64>().ok()).unwrap_or(0);
        let e = Entry { kind, title, url, visits, last };
        let Some(sc) = score(&e, &tokens, now) else { continue };
        let key = dedup_key(url);
        match best.get(&key) {
            Some((old, _)) if *old >= sc => {}
            _ => {
                best.insert(key, (sc, e));
            }
        }
    }

    let mut ranked: Vec<(f64, Entry)> = best.into_values().collect();
    ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.1.url.cmp(b.1.url)));
    ranked.truncate(limit);

    let mut out = String::from("[");
    for (i, (_, e)) in ranked.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str("{\"k\":");
        json::escape_into(&mut out, if e.kind == "b" { "b" } else { "h" });
        out.push_str(",\"title\":");
        json::escape_into(&mut out, e.title);
        out.push_str(",\"url\":");
        json::escape_into(&mut out, e.url);
        out.push('}');
    }
    out.push(']');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA: &str = "\
b\tХабр\thttps://habr.com/ru\t0\t0
h\tХабр: Оптимизация\thttps://habr.com/ru/articles/1\t5\t1000
h\t4PDA форум\thttps://4pda.to/forum\t30\t1000
h\tGitHub\thttps://github.com\t12\t900
h\tПоиск habr на DDG\thttps://duckduckgo.com/?q=habr\t1\t10
bad line without tabs
h\tХабр\thttps://www.habr.com/ru/\t2\t500
";

    #[test]
    fn empty_query_gives_nothing() {
        assert_eq!(suggest("", DATA, 5, 2000), "[]");
        assert_eq!(suggest("   ", DATA, 5, 2000), "[]");
        assert_eq!(suggest("x", DATA, 0, 2000), "[]");
    }

    #[test]
    fn ranks_host_prefix_first_and_dedups() {
        let j = suggest("habr", DATA, 5, 2000);
        let first_habr = j.find("habr.com").unwrap();
        let ddg = j.find("duckduckgo").unwrap();
        assert!(first_habr < ddg, "{j}");
        // habr.com/ru и www.habr.com/ru/ — один результат; всего habr.com/ru, статья и поиск DDG
        assert_eq!(j.matches("\"url\"").count(), 3, "{j}");
    }

    #[test]
    fn all_tokens_must_match() {
        let j = suggest("habr оптим", DATA, 5, 2000);
        assert!(j.contains("articles/1"));
        assert!(!j.contains("github"));
        assert_eq!(suggest("nomatch", DATA, 5, 2000), "[]");
    }

    #[test]
    fn respects_limit_and_case() {
        let j = suggest("ХАБР", DATA, 1, 2000);
        assert_eq!(j.matches("\"url\"").count(), 1);
    }
}
