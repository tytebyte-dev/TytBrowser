//! Лёгкий разбор URL без внешних зависимостей.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed {
    /// Схема в нижнем регистре: `https`, `http`, ...
    pub scheme: String,
    /// Хост в нижнем регистре, без порта и userinfo.
    pub host: String,
    /// Путь + запрос, без фрагмента; всегда начинается с `/`.
    pub path_query: String,
}

pub fn parse(url: &str) -> Option<Parsed> {
    let (scheme, rest) = url.split_once("://")?;
    if scheme.is_empty()
        || !scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    {
        return None;
    }
    let end = rest
        .find(|c| matches!(c, '/' | '?' | '#'))
        .unwrap_or(rest.len());
    let authority = &rest[..end];
    let after = &rest[end..];

    let hostport = authority.rsplit('@').next().unwrap_or(authority);
    let host = if let Some(stripped) = hostport.strip_prefix('[') {
        // IPv6: [::1]:8080
        match stripped.find(']') {
            Some(i) => &stripped[..i],
            None => stripped,
        }
    } else {
        hostport.split(':').next().unwrap_or(hostport)
    };
    if host.is_empty() {
        return None;
    }

    let after = after.split('#').next().unwrap_or("");
    let path_query = if after.starts_with('/') {
        after.to_string()
    } else if after.starts_with('?') {
        format!("/{after}")
    } else {
        "/".to_string()
    };

    Some(Parsed {
        scheme: scheme.to_ascii_lowercase(),
        host: host.trim_end_matches('.').to_lowercase(),
        path_query,
    })
}

pub fn host_of(url: &str) -> String {
    parse(url).map(|p| p.host).unwrap_or_default()
}

fn is_ip(host: &str) -> bool {
    host.contains(':') || (host.split('.').count() == 4 && host.split('.').all(|p| p.parse::<u8>().is_ok()))
}

/// Приближённый «регистрируемый домен» (eTLD+1) без таблицы публичных суффиксов.
/// Достаточно, чтобы отличать стороннюю загрузку от собственной.
pub fn registrable(host: &str) -> String {
    if is_ip(host) {
        return host.to_string();
    }
    let labels: Vec<&str> = host.split('.').filter(|l| !l.is_empty()).collect();
    let n = labels.len();
    if n <= 2 {
        return labels.join(".");
    }
    let tld = labels[n - 1];
    let sld = labels[n - 2];
    let second_level = matches!(
        sld,
        "co" | "com" | "org" | "net" | "gov" | "edu" | "ac" | "or" | "ne" | "go" | "msk" | "spb"
    );
    let take = if tld.len() == 2 && second_level { 3 } else { 2 };
    labels[n - take..].join(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic() {
        let p = parse("HTTPS://User:pw@Sub.Example.COM:8443/a/b?x=1#frag").unwrap();
        assert_eq!(p.scheme, "https");
        assert_eq!(p.host, "sub.example.com");
        assert_eq!(p.path_query, "/a/b?x=1");
    }

    #[test]
    fn parses_edge_cases() {
        assert_eq!(parse("https://example.com").unwrap().path_query, "/");
        assert_eq!(parse("https://example.com?q=1").unwrap().path_query, "/?q=1");
        assert_eq!(parse("http://[::1]:8080/x").unwrap().host, "::1");
        assert!(parse("example.com").is_none());
        assert!(parse("https:///x").is_none());
    }

    #[test]
    fn registrable_domains() {
        assert_eq!(registrable("a.b.example.com"), "example.com");
        assert_eq!(registrable("news.bbc.co.uk"), "bbc.co.uk");
        assert_eq!(registrable("example.com"), "example.com");
        assert_eq!(registrable("192.168.0.1"), "192.168.0.1");
        assert_eq!(registrable("mc.yandex.ru"), "yandex.ru");
    }
}
