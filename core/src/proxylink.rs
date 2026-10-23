//! Разбор ссылок vless:// vmess:// trojan:// ss:// и сборка конфигурации Xray для встроенного прокси.
//! Модуль не зависит от внешних крейтов, поэтому полностью покрывается тестами.

use crate::json;

#[derive(Debug, Clone, PartialEq)]
pub struct ProxyLink {
    pub name: String,
    pub protocol: String,
    pub host: String,
    pub port: u16,
    /// Готовый JSON одного outbound для конфигурации Xray.
    pub outbound: String,
}

fn pct_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = (hex(b[i + 1]), hex(b[i + 2])) {
                out.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// Base64 (обычный и URL-safe алфавиты, с паддингом и без).
pub fn b64_decode(s: &str) -> Option<Vec<u8>> {
    let mut buf = 0u32;
    let mut bits = 0;
    let mut out = Vec::new();
    for c in s.trim().bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' | b'\r' | b'\n' | b' ' => continue,
            _ => return None,
        };
        buf = (buf << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Some(out)
}

fn query(q: &str) -> Vec<(String, String)> {
    q.split('&')
        .filter(|p| !p.is_empty())
        .map(|p| match p.split_once('=') {
            Some((k, v)) => (k.to_string(), pct_decode(v)),
            None => (p.to_string(), String::new()),
        })
        .collect()
}

fn q<'a>(v: &'a [(String, String)], k: &str) -> &'a str {
    v.iter().find(|(a, _)| a == k).map(|(_, b)| b.as_str()).unwrap_or("")
}

/// host:port, включая IPv6 в квадратных скобках.
fn host_port(s: &str) -> Option<(String, u16)> {
    let s = s.trim_end_matches('/');
    let (h, p) = s.rsplit_once(':')?;
    let port: u16 = p.parse().ok()?;
    let host = h.trim_start_matches('[').trim_end_matches(']');
    if host.is_empty() || port == 0 {
        return None;
    }
    Some((host.to_string(), port))
}

struct Stream<'a> {
    network: &'a str,
    security: &'a str,
    sni: &'a str,
    host: &'a str,
    path: &'a str,
    fp: &'a str,
    alpn: &'a str,
    insecure: bool,
    pbk: &'a str,
    sid: &'a str,
    spx: &'a str,
    service: &'a str,
    mode: &'a str,
}

fn stream_json(s: &Stream, fallback_sni: &str) -> Result<String, String> {
    let network = if s.network.is_empty() { "tcp" } else { s.network };
    if !matches!(network, "tcp" | "ws" | "grpc" | "httpupgrade" | "xhttp") {
        return Err(format!(
            "транспорт «{}» не поддерживается (доступны tcp, ws, grpc, httpupgrade, xhttp)",
            network
        ));
    }
    let mut o = format!("{{\"network\":{}", json::string(network));
    let sni = if s.sni.is_empty() { fallback_sni } else { s.sni };
    match s.security {
        "tls" => {
            o.push_str(",\"security\":\"tls\",\"tlsSettings\":{\"serverName\":");
            o.push_str(&json::string(sni));
            o.push_str(&format!(",\"allowInsecure\":{}", s.insecure));
            if !s.alpn.is_empty() {
                let a: Vec<String> = s.alpn.split(',').map(|x| json::string(x.trim())).collect();
                o.push_str(&format!(",\"alpn\":[{}]", a.join(",")));
            }
            if !s.fp.is_empty() {
                o.push_str(&format!(",\"fingerprint\":{}", json::string(s.fp)));
            }
            o.push('}');
        }
        "reality" => {
            if s.pbk.is_empty() {
                return Err("для REALITY нужен параметр pbk (публичный ключ)".into());
            }
            o.push_str(",\"security\":\"reality\",\"realitySettings\":{\"serverName\":");
            o.push_str(&json::string(sni));
            o.push_str(&format!(
                ",\"fingerprint\":{},\"publicKey\":{},\"shortId\":{}",
                json::string(if s.fp.is_empty() { "chrome" } else { s.fp }),
                json::string(s.pbk),
                json::string(s.sid)
            ));
            if !s.spx.is_empty() {
                o.push_str(&format!(",\"spiderX\":{}", json::string(s.spx)));
            }
            o.push('}');
        }
        "" | "none" => {}
        other => return Err(format!("тип защиты «{}» не поддерживается", other)),
    }
    let path = if s.path.is_empty() { "/" } else { s.path };
    let host = if s.host.is_empty() { sni } else { s.host };
    match network {
        "ws" => o.push_str(&format!(
            ",\"wsSettings\":{{\"path\":{},\"headers\":{{\"Host\":{}}}}}",
            json::string(path),
            json::string(host)
        )),
        "grpc" => o.push_str(&format!(",\"grpcSettings\":{{\"serviceName\":{}}}", json::string(s.service))),
        "httpupgrade" => o.push_str(&format!(
            ",\"httpupgradeSettings\":{{\"path\":{},\"host\":{}}}",
            json::string(path),
            json::string(host)
        )),
        "xhttp" => {
            o.push_str(&format!(
                ",\"xhttpSettings\":{{\"path\":{},\"host\":{}",
                json::string(path),
                json::string(host)
            ));
            if !s.mode.is_empty() {
                o.push_str(&format!(",\"mode\":{}", json::string(s.mode)));
            }
            o.push('}');
        }
        _ => {}
    }
    o.push('}');
    Ok(o)
}

fn name_of(frag: Option<&str>, host: &str, port: u16) -> String {
    match frag.map(pct_decode) {
        Some(n) if !n.trim().is_empty() => n.trim().to_string(),
        _ => format!("{}:{}", host, port),
    }
}

fn split_frag(s: &str) -> (&str, Option<&str>) {
    match s.split_once('#') {
        Some((a, b)) => (a, Some(b)),
        None => (s, None),
    }
}

fn parse_vless_trojan(scheme: &str, rest: &str) -> Result<ProxyLink, String> {
    let (body, frag) = split_frag(rest);
    let (main, qs) = body.split_once('?').unwrap_or((body, ""));
    let (cred, hp) = main.rsplit_once('@').ok_or("нет '@' в ссылке")?;
    let (host, port) = host_port(hp).ok_or("некорректные адрес и порт")?;
    let cred = pct_decode(cred);
    let p = query(qs);
    let st = Stream {
        network: q(&p, "type"),
        security: if scheme == "trojan" && q(&p, "security").is_empty() { "tls" } else { q(&p, "security") },
        sni: if q(&p, "sni").is_empty() { q(&p, "peer") } else { q(&p, "sni") },
        host: q(&p, "host"),
        path: q(&p, "path"),
        fp: q(&p, "fp"),
        alpn: q(&p, "alpn"),
        insecure: matches!(q(&p, "allowInsecure"), "1" | "true") || matches!(q(&p, "insecure"), "1" | "true"),
        pbk: q(&p, "pbk"),
        sid: q(&p, "sid"),
        spx: q(&p, "spx"),
        service: q(&p, "serviceName"),
        mode: q(&p, "mode"),
    };
    let stream = stream_json(&st, &host)?;
    let settings = if scheme == "vless" {
        if cred.len() != 36 {
            return Err("UUID должен содержать 36 символов".into());
        }
        format!(
            "{{\"vnext\":[{{\"address\":{},\"port\":{},\"users\":[{{\"id\":{},\"encryption\":\"none\"{}}}]}}]}}",
            json::string(&host), port, json::string(&cred),
            if q(&p, "flow").is_empty() { String::new() } else { format!(",\"flow\":{}", json::string(q(&p, "flow"))) }
        )
    } else {
        format!(
            "{{\"servers\":[{{\"address\":{},\"port\":{},\"password\":{}}}]}}",
            json::string(&host), port, json::string(&cred)
        )
    };
    Ok(ProxyLink {
        name: name_of(frag, &host, port),
        protocol: scheme.to_string(),
        outbound: format!(
            "{{\"tag\":\"proxy\",\"protocol\":\"{}\",\"settings\":{},\"streamSettings\":{}}}",
            scheme, settings, stream
        ),
        host,
        port,
    })
}

/// Достаёт строковое поле верхнего уровня из плоского JSON (формат vmess:// не содержит вложенности).
fn flat_field(js: &str, key: &str) -> String {
    let pat = format!("\"{}\"", key);
    let Some(i) = js.find(&pat) else { return String::new() };
    let rest = js[i + pat.len()..].trim_start();
    let Some(rest) = rest.strip_prefix(':') else { return String::new() };
    let rest = rest.trim_start();
    if let Some(r) = rest.strip_prefix('"') {
        let mut out = String::new();
        let mut it = r.chars();
        while let Some(c) = it.next() {
            match c {
                '"' => break,
                '\\' => match it.next() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some(o) => out.push(o),
                    None => break,
                },
                _ => out.push(c),
            }
        }
        out
    } else {
        rest.chars().take_while(|c| !matches!(c, ',' | '}' | ' ')).collect()
    }
}

fn parse_vmess(rest: &str) -> Result<ProxyLink, String> {
    let (body, _) = split_frag(rest);
    let raw = b64_decode(body).ok_or("vmess: некорректный base64")?;
    let js = String::from_utf8(raw).map_err(|_| "vmess: не UTF-8")?;
    let host = flat_field(&js, "add");
    let port: u16 = flat_field(&js, "port").parse().map_err(|_| "vmess: некорректный порт")?;
    let id = flat_field(&js, "id");
    if host.is_empty() || id.len() != 36 {
        return Err("vmess: нет адреса или UUID".into());
    }
    let tls = flat_field(&js, "tls");
    let st = Stream {
        network: &flat_field(&js, "net"),
        security: if tls == "tls" { "tls" } else { "" },
        sni: &flat_field(&js, "sni"),
        host: &flat_field(&js, "host"),
        path: &flat_field(&js, "path"),
        fp: &flat_field(&js, "fp"),
        alpn: &flat_field(&js, "alpn"),
        insecure: false,
        pbk: "",
        sid: "",
        spx: "",
        service: &flat_field(&js, "path"),
        mode: "",
    };
    let stream = stream_json(&st, &host)?;
    let ps = flat_field(&js, "ps");
    Ok(ProxyLink {
        name: if ps.trim().is_empty() { format!("{}:{}", host, port) } else { ps },
        protocol: "vmess".into(),
        outbound: format!(
            "{{\"tag\":\"proxy\",\"protocol\":\"vmess\",\"settings\":{{\"vnext\":[{{\"address\":{},\"port\":{},\"users\":[{{\"id\":{},\"security\":\"auto\"}}]}}]}},\"streamSettings\":{}}}",
            json::string(&host), port, json::string(&id), stream
        ),
        host,
        port,
    })
}

fn parse_ss(rest: &str) -> Result<ProxyLink, String> {
    let (body, frag) = split_frag(rest);
    let body = body.split('?').next().unwrap_or(body);
    let (userinfo, hp) = if let Some((u, h)) = body.rsplit_once('@') {
        let dec = match b64_decode(u).and_then(|b| String::from_utf8(b).ok()) {
            Some(d) if d.contains(':') => d,
            _ => pct_decode(u),
        };
        (dec, h.to_string())
    } else {
        let dec = b64_decode(body).and_then(|b| String::from_utf8(b).ok()).ok_or("ss: некорректный base64")?;
        let (u, h) = dec.rsplit_once('@').ok_or("ss: нет '@'")?;
        (u.to_string(), h.to_string())
    };
    let (method, password) = userinfo.split_once(':').ok_or("ss: нет метода и пароля")?;
    let (host, port) = host_port(&hp).ok_or("ss: некорректные адрес и порт")?;
    Ok(ProxyLink {
        name: name_of(frag, &host, port),
        protocol: "shadowsocks".into(),
        outbound: format!(
            "{{\"tag\":\"proxy\",\"protocol\":\"shadowsocks\",\"settings\":{{\"servers\":[{{\"address\":{},\"port\":{},\"method\":{},\"password\":{}}}]}}}}",
            json::string(&host), port, json::string(method), json::string(password)
        ),
        host,
        port,
    })
}

pub fn parse(link: &str) -> Result<ProxyLink, String> {
    let link = link.trim();
    let (scheme, rest) = link.split_once("://").ok_or("это не ссылка прокси")?;
    match scheme.to_ascii_lowercase().as_str() {
        "vless" => parse_vless_trojan("vless", rest),
        "trojan" => parse_vless_trojan("trojan", rest),
        "vmess" => parse_vmess(rest),
        "ss" => parse_ss(rest),
        other => Err(format!("схема «{}» не поддерживается", other)),
    }
}

/// Несколько ссылок (по одной в строке) → JSON-массив разобранных профилей: [{name,protocol,host,port,link}], ошибки пропускаются.
pub fn parse_many_json(text: &str) -> String {
    let items: Vec<String> = text
        .split(|c: char| c.is_whitespace())
        .filter(|l| l.contains("://"))
        .filter_map(|l| parse(l).ok().map(|p| (l, p)))
        .map(|(l, p)| {
            format!(
                "{{\"name\":{},\"protocol\":{},\"host\":{},\"port\":{},\"link\":{}}}",
                json::string(&p.name), json::string(&p.protocol), json::string(&p.host), p.port, json::string(l)
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

/// Полная конфигурация Xray: локальный HTTP-прокси на 127.0.0.1:port, весь трафик уходит в outbound `proxy`.
/// В Android нет /etc/resolv.conf, поэтому DNS-серверы заданы явно (адрес сервера прокси может быть доменом).
pub fn build_config(link: &str, listen_port: u16) -> Result<String, String> {
    let p = parse(link)?;
    Ok(format!(
        "{{\"log\":{{\"loglevel\":\"warning\"}},\"dns\":{{\"servers\":[\"1.1.1.1\",\"8.8.8.8\",\"localhost\"]}},\"inbounds\":[{{\"tag\":\"in\",\"port\":{},\"listen\":\"127.0.0.1\",\"protocol\":\"http\",\"settings\":{{}}}}],\"outbounds\":[{}]}}",
        listen_port, p.outbound
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const U: &str = "11111111-2222-3333-4444-555555555555";

    #[test]
    fn vless_reality_and_flow() {
        let l = format!("vless://{}@1.2.3.4:443?type=tcp&security=reality&sni=ex.com&pbk=KEY&sid=ab&flow=xtls-rprx-vision", U);
        let r = parse(&l).unwrap();
        assert!(r.outbound.contains("\"publicKey\":\"KEY\"") && r.outbound.contains("\"shortId\":\"ab\""));
        assert!(r.outbound.contains("\"fingerprint\":\"chrome\"") && r.outbound.contains("\"encryption\":\"none\""));
        let l = format!("vless://{}@1.2.3.4:443?security=reality&sni=a", U);
        assert!(parse(&l).is_err());
        let l = format!("vless://{}@1.2.3.4:443?security=tls&flow=xtls-rprx-vision#My%20Node", U);
        let p = parse(&l).unwrap();
        assert_eq!(p.name, "My Node");
        assert!(p.outbound.contains("\"flow\":\"xtls-rprx-vision\""));
    }

    #[test]
    fn vless_ws_tls() {
        let l = format!("vless://{}@h.example:8443?type=ws&security=tls&path=%2Fws&host=cdn.x&sni=s.x#n", U);
        let p = parse(&l).unwrap();
        assert!(p.outbound.contains("\"path\":\"/ws\"") && p.outbound.contains("\"Host\":\"cdn.x\""));
        assert!(p.outbound.contains("\"serverName\":\"s.x\""));
    }

    #[test]
    fn unsupported_transport() {
        let l = format!("vless://{}@h:443?type=kcp&security=tls", U);
        assert!(parse(&l).unwrap_err().contains("kcp"));
        let l = format!("vless://{}@h:443?type=grpc&security=tls&serviceName=svc", U);
        assert!(parse(&l).unwrap().outbound.contains("\"serviceName\":\"svc\""));
        let l = format!("vless://{}@h:443?type=xhttp&security=tls&path=%2Fx&mode=auto", U);
        assert!(parse(&l).unwrap().outbound.contains("\"mode\":\"auto\""));
    }

    #[test]
    fn trojan_defaults_to_tls() {
        let p = parse("trojan://pa%24s@t.example:443?sni=t.example#T").unwrap();
        assert!(p.outbound.contains("\"password\":\"pa$s\""));
        assert!(p.outbound.contains("\"security\":\"tls\""));
    }

    #[test]
    fn vmess_base64() {
        let js = format!("{{\"v\":\"2\",\"ps\":\"vm\",\"add\":\"9.9.9.9\",\"port\":\"443\",\"id\":\"{}\",\"net\":\"ws\",\"path\":\"/p\",\"tls\":\"tls\",\"host\":\"h\"}}", U);
        let enc = b64_encode(js.as_bytes());
        let p = parse(&format!("vmess://{}", enc)).unwrap();
        assert_eq!(p.name, "vm");
        assert_eq!(p.port, 443);
        assert!(p.outbound.contains("\"path\":\"/p\""));
    }

    #[test]
    fn ss_sip002_and_legacy() {
        let u = b64_encode(b"aes-256-gcm:secret");
        let p = parse(&format!("ss://{}@5.6.7.8:8388#ss1", u)).unwrap();
        assert_eq!(p.name, "ss1");
        assert!(p.outbound.contains("\"method\":\"aes-256-gcm\""));
        let all = b64_encode(b"chacha20-ietf-poly1305:pw@[::1]:1080");
        let p = parse(&format!("ss://{}", all)).unwrap();
        assert_eq!((p.host.as_str(), p.port), ("::1", 1080));
    }

    #[test]
    fn config_and_many() {
        let l = format!("vless://{}@1.2.3.4:443?security=tls#a", U);
        let c = build_config(&l, 10808).unwrap();
        assert!(c.contains("\"port\":10808") && c.contains("\"protocol\":\"http\""));
        let txt = format!("{}\njunk\nss://{}@1.1.1.1:1#b", l, b64_encode(b"aes-128-gcm:x"));
        let m = parse_many_json(&txt);
        assert!(m.starts_with("[{") && m.contains("\"name\":\"b\""));
        assert!(parse("http://x").is_err());
    }

    fn b64_encode(d: &[u8]) -> String {
        const A: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut o = String::new();
        for c in d.chunks(3) {
            let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
            o.push(A[(n >> 18) as usize & 63] as char);
            o.push(A[(n >> 12) as usize & 63] as char);
            if c.len() > 1 { o.push(A[(n >> 6) as usize & 63] as char) } else { o.push('=') }
            if c.len() > 2 { o.push(A[n as usize & 63] as char) } else { o.push('=') }
        }
        o
    }
}
