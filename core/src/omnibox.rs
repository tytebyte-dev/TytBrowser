//! Разбор ввода в адресной строке: адрес, поисковый запрос или внутренняя страница.
//! Плюс очистка ссылок от трекинговых параметров.

use crate::json;

pub const START_URL: &str = "tyt://start";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Internal,
    Url,
    Search,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Internal => "internal",
            Kind::Url => "url",
            Kind::Search => "search",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    pub kind: Kind,
    pub url: String,
    pub query: String,
}

impl Resolved {
    pub fn to_json(&self) -> String {
        let mut s = String::from("{\"kind\":");
        json::escape_into(&mut s, self.kind.as_str());
        s.push_str(",\"url\":");
        json::escape_into(&mut s, &self.url);
        s.push_str(",\"query\":");
        json::escape_into(&mut s, &self.query);
        s.push('}');
        s
    }
}

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Адрес поиска для выбранной системы (`duckduckgo`, `google`, `yandex`, `bing`).
pub fn search_url(engine: &str, query: &str) -> String {
    let q = percent_encode(query.trim());
    match engine {
        "google" => format!("https://www.google.com/search?q={q}"),
        "yandex" => format!("https://yandex.ru/search/?text={q}"),
        "bing" => format!("https://www.bing.com/search?q={q}"),
        _ => format!("https://duckduckgo.com/?q={q}"),
    }
}

fn internal_start() -> Resolved {
    Resolved {
        kind: Kind::Internal,
        url: START_URL.to_string(),
        query: String::new(),
    }
}

fn search(engine: &str, q: &str) -> Resolved {
    let q = q.trim();
    if q.is_empty() {
        return internal_start();
    }
    Resolved {
        kind: Kind::Search,
        url: search_url(engine, q),
        query: q.to_string(),
    }
}

fn is_private_ipv4(host: &str) -> Option<bool> {
    let parts: Vec<u8> = host
        .split('.')
        .map(|p| p.parse::<u8>().ok())
        .collect::<Option<Vec<_>>>()?;
    if parts.len() != 4 {
        return None;
    }
    Some(
        parts[0] == 10
            || parts[0] == 127
            || (parts[0] == 192 && parts[1] == 168)
            || (parts[0] == 172 && (16..=31).contains(&parts[1])),
    )
}

/// Похоже ли `s` на адрес сайта без схемы. Возвращает `Some(использовать_http)`.
fn looks_like_host(s: &str) -> Option<bool> {
    let end = s.find(|c| matches!(c, '/' | '?' | '#')).unwrap_or(s.len());
    let authority = &s[..end];
    if authority.is_empty() || authority.contains('@') {
        return None;
    }

    // IPv6 в скобках: [::1] или [::1]:8080
    if let Some(rest) = authority.strip_prefix('[') {
        let close = rest.find(']')?;
        let inner = &rest[..close];
        let tail = &rest[close + 1..];
        let ok = !inner.is_empty()
            && inner.chars().all(|c| c.is_ascii_hexdigit() || c == ':' || c == '.')
            && (tail.is_empty() || (tail.starts_with(':') && tail[1..].chars().all(|c| c.is_ascii_digit())));
        return if ok { Some(true) } else { None };
    }

    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => (h, Some(p)),
        None => (authority, None),
    };
    if let Some(p) = port {
        if p.is_empty() || p.len() > 5 || !p.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
    }
    if host.is_empty() {
        return None;
    }
    if host.eq_ignore_ascii_case("localhost") {
        return Some(true);
    }
    if let Some(private) = is_private_ipv4(host) {
        return Some(private);
    }

    if !host.contains('.') {
        return None;
    }
    let labels: Vec<&str> = host.split('.').collect();
    for l in &labels {
        if l.is_empty()
            || l.starts_with('-')
            || l.ends_with('-')
            || !l.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return None;
        }
    }
    let tld = labels[labels.len() - 1];
    let tld_ok = tld.to_ascii_lowercase().starts_with("xn--")
        || (tld.chars().count() >= 2 && tld.chars().all(|c| c.is_alphabetic()));
    if tld_ok {
        Some(false)
    } else {
        None
    }
}

const BLOCKED_PREFIXES: &[&str] = &[
    "javascript:", "data:", "vbscript:", "file:", "content:", "intent:", "blob:", "about:",
    "view-source:", "chrome:", "mailto:", "tel:", "sms:", "geo:",
];

/// Главная функция омнибокса.
pub fn resolve(input: &str, engine: &str) -> Resolved {
    let s = input.trim();
    if s.is_empty() {
        return internal_start();
    }
    let lower = s.to_lowercase();

    if matches!(lower.as_str(), "start" | "about:blank" | "about:home" | "tyt://start") {
        return internal_start();
    }
    if lower.starts_with("tyt://") {
        return Resolved {
            kind: Kind::Internal,
            url: lower,
            query: String::new(),
        };
    }
    if let Some(q) = s.strip_prefix('?') {
        return search(engine, q);
    }
    if lower.starts_with("http://") || lower.starts_with("https://") {
        let has_host = crate::url_util::parse(s).is_some();
        if has_host && !s.chars().any(char::is_whitespace) {
            return Resolved {
                kind: Kind::Url,
                url: s.to_string(),
                query: String::new(),
            };
        }
        return search(engine, s);
    }
    // Любые другие схемы (javascript:, data:, intent:, ftp://, ...) — не навигация, а запрос.
    if BLOCKED_PREFIXES.iter().any(|p| lower.starts_with(p)) || lower.contains("://") {
        return search(engine, s);
    }
    if s.chars().any(char::is_whitespace) {
        return search(engine, s);
    }
    match looks_like_host(s) {
        Some(use_http) => Resolved {
            kind: Kind::Url,
            url: format!("{}://{}", if use_http { "http" } else { "https" }, s),
            query: String::new(),
        },
        None => search(engine, s),
    }
}

fn is_tracking_param(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    n.starts_with("utm_")
        || matches!(
            n.as_str(),
            "fbclid" | "gclid" | "dclid" | "gbraid" | "wbraid" | "yclid" | "ysclid" | "msclkid"
                | "twclid" | "igshid" | "mc_cid" | "mc_eid" | "_hsenc" | "_hsmi" | "mkt_tok"
                | "vero_id" | "oly_enc_id" | "oly_anon_id"
        )
}

/// Убирает из http(s)-ссылки трекинговые параметры (`utm_*`, `fbclid`, `gclid`, ...).
/// Если ничего не удалено — возвращает исходную строку без изменений.
pub fn clean_url(url: &str) -> String {
    let lower = url.get(..8).unwrap_or(url).to_ascii_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return url.to_string();
    }
    let (before_frag, frag) = match url.split_once('#') {
        Some((a, b)) => (a, Some(b)),
        None => (url, None),
    };
    let Some((base, query)) = before_frag.split_once('?') else {
        return url.to_string();
    };
    let kept: Vec<&str> = query
        .split('&')
        .filter(|p| !p.is_empty())
        .filter(|p| !is_tracking_param(p.split('=').next().unwrap_or("")))
        .collect();
    if kept.len() == query.split('&').filter(|p| !p.is_empty()).count() {
        return url.to_string();
    }
    let mut out = base.to_string();
    if !kept.is_empty() {
        out.push('?');
        out.push_str(&kept.join("&"));
    }
    if let Some(f) = frag {
        out.push('#');
        out.push_str(f);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(s: &str) -> Resolved {
        resolve(s, "duckduckgo")
    }

    #[test]
    fn empty_and_start() {
        assert_eq!(r("").kind, Kind::Internal);
        assert_eq!(r("  start ").url, START_URL);
        assert_eq!(r("about:blank").url, START_URL);
        assert_eq!(r("tyt://start").url, START_URL);
    }

    #[test]
    fn plain_domains_get_https() {
        assert_eq!(r("habr.com").url, "https://habr.com");
        assert_eq!(r("habr.com/ru/articles/1?x=1").url, "https://habr.com/ru/articles/1?x=1");
        assert_eq!(r("пример.рф").url, "https://пример.рф");
        assert_eq!(r("sub.site.io:8443/a").kind, Kind::Url);
    }

    #[test]
    fn local_hosts_use_http() {
        assert_eq!(r("localhost:3000").url, "http://localhost:3000");
        assert_eq!(r("192.168.1.1").url, "http://192.168.1.1");
        assert_eq!(r("8.8.8.8").url, "https://8.8.8.8");
        assert_eq!(r("[::1]:8080").url, "http://[::1]:8080");
    }

    #[test]
    fn full_urls_pass_through() {
        assert_eq!(r("https://example.com/a b").kind, Kind::Search);
        assert_eq!(r("HTTP://Example.com").url, "HTTP://Example.com");
        assert_eq!(r("https://").kind, Kind::Search);
    }

    #[test]
    fn searches() {
        let s = r("как настроить android");
        assert_eq!(s.kind, Kind::Search);
        assert_eq!(s.query, "как настроить android");
        assert!(s.url.starts_with("https://duckduckgo.com/?q=%D0%BA"));
        assert_eq!(r("android").kind, Kind::Search);
        assert_eq!(r("?habr.com").kind, Kind::Search);
        assert_eq!(r("3.14").kind, Kind::Search);
        assert_eq!(r("e.g.").kind, Kind::Search);
        assert_eq!(r("user@example.com").kind, Kind::Search);
    }

    #[test]
    fn dangerous_schemes_become_searches() {
        for s in ["javascript:alert(1)", "data:text/html,<b>x", "file:///etc/passwd", "intent://x#Intent;end", "ftp://x.org"] {
            assert_eq!(r(s).kind, Kind::Search, "{s}");
        }
    }

    #[test]
    fn engines() {
        assert!(resolve("a b", "google").url.starts_with("https://www.google.com/search?q=a%20b"));
        assert!(resolve("a b", "yandex").url.starts_with("https://yandex.ru/search/?text=a%20b"));
        assert!(resolve("a b", "bing").url.starts_with("https://www.bing.com/search?q="));
        assert!(resolve("a b", "unknown").url.starts_with("https://duckduckgo.com/"));
    }

    #[test]
    fn json_output() {
        let j = r("a\"b c").to_json();
        assert!(j.starts_with("{\"kind\":\"search\""));
        assert!(j.contains("\"query\":\"a\\\"b c\""));
    }

    #[test]
    fn cleans_tracking_params() {
        assert_eq!(
            clean_url("https://x.com/a?utm_source=t&id=5&fbclid=abc#top"),
            "https://x.com/a?id=5#top"
        );
        assert_eq!(clean_url("https://x.com/a?utm_source=t&gclid=1"), "https://x.com/a");
        assert_eq!(clean_url("https://x.com/a?id=5"), "https://x.com/a?id=5");
        assert_eq!(clean_url("tyt://start"), "tyt://start");
        assert_eq!(clean_url("https://x.com"), "https://x.com");
    }
}
