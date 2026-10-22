//! Блокировщик рекламы и трекеров.
//!
//! Поддерживаемый синтаксис правил (подмножество Adblock Plus + hosts):
//!   * `||example.com^`             — домен и все поддомены
//!   * `||example.com/path^`        — только запросы с этим префиксом пути
//!   * `||example.com^$third-party` — только сторонние запросы
//!   * `@@||example.com^`           — исключение
//!   * `0.0.0.0 example.com`        — строка формата hosts
//!   * `example.com`                — просто домен
//! Всё остальное (wildcard, регулярные выражения, косметика) молча пропускается.

use crate::url_util;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Verdict {
    Allow = 0,
    Ads = 1,
    Trackers = 2,
}

#[derive(Debug, Clone)]
struct PathPat {
    text: String,
    /// После шаблона должен быть разделитель (`^`) или конец строки.
    sep: bool,
}

impl PathPat {
    fn matches(&self, path_query: &str) -> bool {
        let Some(rest) = path_query.strip_prefix(self.text.as_str()) else {
            return false;
        };
        if !self.sep {
            return true;
        }
        match rest.chars().next() {
            None => true,
            Some(c) => !(c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '%')),
        }
    }
}

#[derive(Debug, Clone)]
struct Rule {
    path: Option<PathPat>,
    third_party_only: bool,
}

#[derive(Default)]
struct RuleSet {
    map: HashMap<String, Vec<Rule>>,
    count: usize,
}

impl RuleSet {
    fn add(&mut self, domain: String, rule: Rule) {
        self.map.entry(domain).or_default().push(rule);
        self.count += 1;
    }

    fn matches(&self, host: &str, path_query: &str, third_party: bool) -> bool {
        if self.map.is_empty() {
            return false;
        }
        let mut suffix = host;
        loop {
            if let Some(rules) = self.map.get(suffix) {
                for r in rules {
                    if r.third_party_only && !third_party {
                        continue;
                    }
                    match &r.path {
                        None => return true,
                        Some(p) if p.matches(path_query) => return true,
                        Some(_) => {}
                    }
                }
            }
            match suffix.find('.') {
                Some(i) => suffix = &suffix[i + 1..],
                None => return false,
            }
        }
    }
}

#[derive(Default)]
pub struct Blocker {
    ads: RuleSet,
    trackers: RuleSet,
    allow: RuleSet,
    pub ads_on: bool,
    pub trackers_on: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Ads,
    Trackers,
}

fn valid_domain(d: &str) -> bool {
    d.contains('.')
        && !d.starts_with('.')
        && !d.ends_with('.')
        && d.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
}

enum Parsed {
    Block(String, Rule),
    Allow(String, Rule),
}

fn parse_line(line: &str) -> Option<Parsed> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('!') || line.starts_with('#') || line.starts_with('[') {
        return None;
    }

    // hosts: "0.0.0.0 example.com" / "127.0.0.1 example.com"
    let mut parts = line.split_whitespace();
    if let (Some(a), Some(b), None) = (parts.next(), parts.next(), parts.next()) {
        if a == "0.0.0.0" || a == "127.0.0.1" {
            let d = b.to_ascii_lowercase();
            return if valid_domain(&d) {
                Some(Parsed::Block(d, Rule { path: None, third_party_only: false }))
            } else {
                None
            };
        }
    }
    if line.contains(char::is_whitespace) {
        return None;
    }

    let (is_allow, body) = match line.strip_prefix("@@") {
        Some(rest) => (true, rest),
        None => (false, line),
    };

    // Просто домен.
    if !body.starts_with("||") {
        let d = body.to_ascii_lowercase();
        return if valid_domain(&d) {
            let rule = Rule { path: None, third_party_only: false };
            Some(if is_allow { Parsed::Allow(d, rule) } else { Parsed::Block(d, rule) })
        } else {
            None
        };
    }

    let body = &body[2..];
    let (pattern, options) = match body.rsplit_once('$') {
        Some((p, o)) => (p, Some(o)),
        None => (body, None),
    };
    let mut third_party_only = false;
    if let Some(opts) = options {
        for o in opts.split(',') {
            match o {
                "third-party" | "3p" => third_party_only = true,
                "important" | "all" => {}
                _ => return None, // типы ресурсов, domain=..., и т.д. не поддерживаем
            }
        }
    }
    if pattern.contains('*') {
        return None;
    }

    let dom_end = pattern
        .find(|c| matches!(c, '^' | '/' | '?' | ':' | '|'))
        .unwrap_or(pattern.len());
    let domain = pattern[..dom_end].to_ascii_lowercase();
    if !valid_domain(&domain) {
        return None;
    }
    let rest = &pattern[dom_end..];

    let path = if rest.starts_with('/') {
        let sep = rest.ends_with('^');
        let text = rest.trim_end_matches(['^', '|']).to_string();
        if text.is_empty() {
            None
        } else {
            Some(PathPat { text, sep })
        }
    } else {
        None
    };

    let rule = Rule { path, third_party_only };
    Some(if is_allow { Parsed::Allow(domain, rule) } else { Parsed::Block(domain, rule) })
}

impl Blocker {
    pub fn new() -> Self {
        Blocker { ads_on: true, trackers_on: true, ..Default::default() }
    }

    pub fn load_defaults(&mut self) {
        self.add_rules(Category::Ads, include_str!("../rules/ads.txt"));
        self.add_rules(Category::Trackers, include_str!("../rules/trackers.txt"));
    }

    /// Добавляет правила из текста; возвращает число принятых правил.
    pub fn add_rules(&mut self, cat: Category, text: &str) -> usize {
        let mut added = 0;
        for line in text.lines() {
            match parse_line(line) {
                Some(Parsed::Block(d, r)) => {
                    match cat {
                        Category::Ads => self.ads.add(d, r),
                        Category::Trackers => self.trackers.add(d, r),
                    }
                    added += 1;
                }
                Some(Parsed::Allow(d, r)) => {
                    self.allow.add(d, r);
                    added += 1;
                }
                None => {}
            }
        }
        added
    }

    pub fn rule_count(&self) -> usize {
        self.ads.count + self.trackers.count + self.allow.count
    }

    /// Решение для подресурса `url`, загружаемого со страницы `page_url`.
    pub fn check(&self, url: &str, page_url: &str) -> Verdict {
        if !self.ads_on && !self.trackers_on {
            return Verdict::Allow;
        }
        let Some(p) = url_util::parse(url) else {
            return Verdict::Allow;
        };
        if !matches!(p.scheme.as_str(), "http" | "https" | "ws" | "wss") {
            return Verdict::Allow;
        }
        let page_host = url_util::host_of(page_url);
        let third = !page_host.is_empty()
            && url_util::registrable(&p.host) != url_util::registrable(&page_host);

        if self.allow.matches(&p.host, &p.path_query, third) {
            return Verdict::Allow;
        }
        if self.ads_on && self.ads.matches(&p.host, &p.path_query, third) {
            return Verdict::Ads;
        }
        if self.trackers_on && self.trackers.matches(&p.host, &p.path_query, third) {
            return Verdict::Trackers;
        }
        Verdict::Allow
    }
}

static GLOBAL: OnceLock<RwLock<Blocker>> = OnceLock::new();

/// Глобальный экземпляр с встроенными списками (создаётся при первом обращении).
pub fn global() -> &'static RwLock<Blocker> {
    GLOBAL.get_or_init(|| {
        let mut b = Blocker::new();
        b.load_defaults();
        RwLock::new(b)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blocker() -> Blocker {
        let mut b = Blocker::new();
        b.load_defaults();
        b
    }

    #[test]
    fn defaults_load() {
        let b = blocker();
        assert!(b.rule_count() > 100, "rules: {}", b.rule_count());
    }

    #[test]
    fn blocks_ads_and_subdomains() {
        let b = blocker();
        let page = "https://news.example.org/a";
        assert_eq!(b.check("https://doubleclick.net/x.js", page), Verdict::Ads);
        assert_eq!(b.check("https://securepubads.g.doubleclick.net/gpt.js", page), Verdict::Ads);
        assert_eq!(b.check("https://example.org/app.js", page), Verdict::Allow);
        assert_eq!(b.check("https://notdoubleclick.net/x.js", page), Verdict::Allow);
    }

    #[test]
    fn blocks_trackers() {
        let b = blocker();
        let page = "https://shop.example.com/";
        assert_eq!(b.check("https://mc.yandex.ru/metrika/tag.js", page), Verdict::Trackers);
        assert_eq!(b.check("https://www.google-analytics.com/analytics.js", page), Verdict::Trackers);
    }

    #[test]
    fn toggles_are_independent() {
        let mut b = blocker();
        let page = "https://example.com/";
        b.ads_on = false;
        assert_eq!(b.check("https://doubleclick.net/x", page), Verdict::Allow);
        assert_eq!(b.check("https://mc.yandex.ru/x", page), Verdict::Trackers);
        b.trackers_on = false;
        assert_eq!(b.check("https://mc.yandex.ru/x", page), Verdict::Allow);
    }

    #[test]
    fn path_and_third_party_rules() {
        let mut b = Blocker::new();
        b.add_rules(
            Category::Trackers,
            "||facebook.com/tr^$third-party\n||site.com/ads/\n||cdn.com^$third-party\n",
        );
        let other = "https://blog.example.com/";
        assert_eq!(b.check("https://www.facebook.com/tr?id=1", other), Verdict::Trackers);
        assert_eq!(b.check("https://www.facebook.com/trending", other), Verdict::Allow);
        // собственный запрос facebook.com со страницы facebook.com — не сторонний
        assert_eq!(b.check("https://www.facebook.com/tr?id=1", "https://www.facebook.com/"), Verdict::Allow);
        assert_eq!(b.check("https://site.com/ads/banner.png", other), Verdict::Trackers);
        assert_eq!(b.check("https://site.com/adsense/", other), Verdict::Allow);
        assert_eq!(b.check("https://cdn.com/a.js", other), Verdict::Trackers);
        assert_eq!(b.check("https://cdn.com/a.js", "https://cdn.com/"), Verdict::Allow);
    }

    #[test]
    fn exceptions_win() {
        let mut b = Blocker::new();
        b.add_rules(Category::Ads, "||ads.example.net^\n@@||ads.example.net/ok^\n");
        let page = "https://example.com/";
        assert_eq!(b.check("https://ads.example.net/banner", page), Verdict::Ads);
        assert_eq!(b.check("https://ads.example.net/ok/x", page), Verdict::Allow);
    }

    #[test]
    fn parses_hosts_and_comments() {
        let mut b = Blocker::new();
        let n = b.add_rules(
            Category::Ads,
            "# comment\n! comment\n[Adblock Plus 2.0]\n0.0.0.0 bad.example.com\n127.0.0.1 localhost\nplain.example.org\n||wild*.com^\n||x.com^$script\n\n",
        );
        assert_eq!(n, 2);
        assert_eq!(b.check("https://bad.example.com/a", "https://z.org/"), Verdict::Ads);
        assert_eq!(b.check("https://plain.example.org/a", "https://z.org/"), Verdict::Ads);
    }

    #[test]
    fn ignores_non_http() {
        let b = blocker();
        assert_eq!(b.check("data:image/png;base64,AAAA", "https://a.com/"), Verdict::Allow);
        assert_eq!(b.check("not a url", "https://a.com/"), Verdict::Allow);
    }
}
