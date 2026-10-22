// Module: common\utils\browser.rs
// 1:1 Rust implementation corresponding to Go common\utils\browser.go

use std::collections::HashMap;

pub const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36";
pub const FIREFOX_UA: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:140.0) Gecko/20100101 Firefox/140.0";

pub fn is_browser_user_agent(ua: &str) -> bool {
    ua.contains("Mozilla")
        || ua.contains("Chrome")
        || ua.contains("Safari")
        || ua.contains("Firefox")
}

pub fn chrome_version() -> u32 {
    // Starting version: Chrome 144

    144u32
}

const CLIENT_HINT_GREASE_NA: &[&str] = &[" ", "(", ":", "-", ".", "/", ")", ";", "=", "?", "_"];
const CLIENT_HINT_VERSION_NA: &[&str] = &["8", "99", "24"];
const CLIENT_HINT_SHUFFLE_3: &[[usize; 3]] = &[
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
];
const CLIENT_HINT_SHUFFLE_4: &[[usize; 4]] = &[
    [0, 1, 2, 3],
    [0, 1, 3, 2],
    [0, 2, 1, 3],
    [0, 2, 3, 1],
    [0, 3, 1, 2],
    [0, 3, 2, 1],
    [1, 0, 2, 3],
    [1, 0, 3, 2],
    [1, 2, 0, 3],
    [1, 2, 3, 0],
    [1, 3, 0, 2],
    [1, 3, 2, 0],
    [2, 0, 1, 3],
    [2, 0, 3, 1],
    [2, 1, 0, 3],
    [2, 1, 3, 0],
    [2, 3, 0, 1],
    [2, 3, 1, 0],
    [3, 0, 1, 2],
    [3, 0, 2, 1],
    [3, 1, 0, 2],
    [3, 1, 2, 0],
    [3, 2, 0, 1],
    [3, 2, 1, 0],
];

pub fn get_greased_ch_invalid_brand(seed: usize) -> String {
    let g1 = CLIENT_HINT_GREASE_NA[seed % CLIENT_HINT_GREASE_NA.len()];
    let g2 = CLIENT_HINT_GREASE_NA[(seed + 1) % CLIENT_HINT_GREASE_NA.len()];
    let ver = CLIENT_HINT_VERSION_NA[seed % CLIENT_HINT_VERSION_NA.len()];
    format!("\"Not{}A{}Brand\";v=\"{}\"", g1, g2, ver)
}

pub fn get_greased_ch_order(brand_length: usize, seed: usize) -> Vec<usize> {
    match brand_length {
        1 => vec![0],
        2 => vec![seed % brand_length, (seed + 1) % brand_length],
        3 => CLIENT_HINT_SHUFFLE_3[seed % CLIENT_HINT_SHUFFLE_3.len()].to_vec(),
        _ => CLIENT_HINT_SHUFFLE_4[seed % CLIENT_HINT_SHUFFLE_4.len()].to_vec(),
    }
}

pub fn get_ungreased_ch_ua(major_version: u32, fork_name: &str) -> Vec<String> {
    let mut base = Vec::with_capacity(4);
    base.push(get_greased_ch_invalid_brand(major_version as usize));
    base.push(format!("\"Chromium\";v=\"{}\"", major_version));
    match fork_name {
        "chrome" => base.push(format!("\"Google Chrome\";v=\"{}\"", major_version)),
        "edge" => base.push(format!("\"Microsoft Edge\";v=\"{}\"", major_version)),
        _ => {}
    }
    base
}

pub fn get_greased_ch_ua(major_version: u32, fork_name: &str) -> String {
    let ungreased = get_ungreased_ch_ua(major_version, fork_name);
    let order = get_greased_ch_order(ungreased.len(), major_version as usize);
    let mut shuffled = vec![String::new(); ungreased.len()];
    for (i, &pos) in order.iter().enumerate() {
        if pos < shuffled.len() && i < ungreased.len() {
            shuffled[pos] = ungreased[i].clone();
        }
    }
    shuffled.join(", ")
}

pub fn chrome_ua() -> String {
    let ver = chrome_version();
    format!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36",
        ver
    )
}

pub fn chrome_uach() -> String {
    get_greased_ch_ua(chrome_version(), "chrome")
}

pub fn msedge_ua() -> String {
    let ver = chrome_version();
    format!(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{}.0.0.0 Safari/537.36 Edg/{}.0.0.0",
        ver, ver
    )
}

pub fn msedge_uach() -> String {
    get_greased_ch_ua(chrome_version(), "edge")
}

pub fn apply_masqueraded_headers(
    headers: &mut HashMap<String, String>,
    browser: &str,
    variant: &str,
) {
    match browser {
        "chrome" => {
            headers.insert("Sec-CH-UA".into(), chrome_uach());
            headers.insert("Sec-CH-UA-Mobile".into(), "?0".into());
            headers.insert("Sec-CH-UA-Platform".into(), "\"Windows\"".into());
            headers.insert("DNT".into(), "1".into());
            headers.insert("User-Agent".into(), chrome_ua());
            headers.insert("Accept-Language".into(), "en-US,en;q=0.9".into());
        }
        "edge" => {
            headers.insert("Sec-CH-UA".into(), msedge_uach());
            headers.insert("Sec-CH-UA-Mobile".into(), "?0".into());
            headers.insert("Sec-CH-UA-Platform".into(), "\"Windows\"".into());
            headers.insert("DNT".into(), "1".into());
            headers.insert("User-Agent".into(), msedge_ua());
            headers.insert("Accept-Language".into(), "en-US,en;q=0.9".into());
        }
        "firefox" => {
            headers.insert("User-Agent".into(), FIREFOX_UA.into());
            headers.insert("DNT".into(), "1".into());
            headers.insert("Accept-Language".into(), "en-US,en;q=0.5".into());
        }
        "golang" => {
            headers.remove("User-Agent");
            return;
        }
        _ => {}
    }

    match variant {
        "nav" => {
            if !headers.contains_key("Cache-Control") && (browser == "chrome" || browser == "edge")
            {
                headers.insert("Cache-Control".into(), "max-age=0".into());
            }
            headers.insert("Upgrade-Insecure-Requests".into(), "1".into());
            if !headers.contains_key("Accept") {
                match browser {
                    "chrome" | "edge" => {
                        headers.insert(
                            "Accept".into(),
                            "text/html,application/xhtml+xml,application/xml;q=0.9,image/jxl,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7".into(),
                        );
                    }
                    "firefox" => {
                        headers.insert(
                            "Accept".into(),
                            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
                                .into(),
                        );
                    }
                    _ => {}
                }
            }
            headers.insert("Sec-Fetch-Site".into(), "none".into());
            headers.insert("Sec-Fetch-Mode".into(), "navigate".into());
            headers.insert("Sec-Fetch-User".into(), "?1".into());
            headers.insert("Sec-Fetch-Dest".into(), "document".into());
            headers.insert("Priority".into(), "u=0, i".into());
        }
        "ws" => {
            headers.insert("Sec-Fetch-Mode".into(), "websocket".into());
            headers.insert("Sec-Fetch-Dest".into(), "empty".into());
            headers.insert("Sec-Fetch-Site".into(), "same-origin".into());
            headers
                .entry("Cache-Control".into())
                .or_insert_with(|| "no-cache".into());
            headers
                .entry("Pragma".into())
                .or_insert_with(|| "no-cache".into());
            headers
                .entry("Accept".into())
                .or_insert_with(|| "*/*".into());
        }
        "fetch" => {
            headers.insert("Sec-Fetch-Mode".into(), "cors".into());
            headers.insert("Sec-Fetch-Dest".into(), "empty".into());
            headers.insert("Sec-Fetch-Site".into(), "same-origin".into());
            if !headers.contains_key("Priority") {
                match browser {
                    "chrome" | "edge" => {
                        headers.insert("Priority".into(), "u=1, i".into());
                    }
                    "firefox" => {
                        headers.insert("Priority".into(), "u=4".into());
                    }
                    _ => {}
                }
            }
            headers
                .entry("Cache-Control".into())
                .or_insert_with(|| "no-cache".into());
            headers
                .entry("Pragma".into())
                .or_insert_with(|| "no-cache".into());
            headers
                .entry("Accept".into())
                .or_insert_with(|| "*/*".into());
        }
        _ => {}
    }
}

pub fn try_default_headers_with(headers: &mut HashMap<String, String>, variant: &str) {
    let ua = headers.get("User-Agent").cloned();
    match ua.as_deref() {
        None | Some("") => apply_masqueraded_headers(headers, "chrome", variant),
        Some("chrome") => apply_masqueraded_headers(headers, "chrome", variant),
        Some("firefox") => apply_masqueraded_headers(headers, "firefox", variant),
        Some("edge") => apply_masqueraded_headers(headers, "edge", variant),
        Some("golang") => apply_masqueraded_headers(headers, "golang", variant),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chrome_version_and_ua() {
        let ver = chrome_version();
        assert!(ver >= 144);
        let ua = chrome_ua();
        assert!(ua.contains(&format!("Chrome/{}.0.0.0", ver)));
        assert!(is_browser_user_agent(&ua));
    }

    #[test]
    fn test_greased_ch_ua() {
        let ch = chrome_uach();
        assert!(ch.contains("Chromium"));
        assert!(ch.contains("Google Chrome"));
        assert!(ch.contains("Not"));
    }

    #[test]
    fn test_apply_masqueraded_headers() {
        let mut headers = HashMap::new();
        try_default_headers_with(&mut headers, "nav");
        assert!(headers.contains_key("User-Agent"));
        assert!(headers.contains_key("Sec-CH-UA"));
        assert_eq!(headers.get("Sec-Fetch-Mode"), Some(&"navigate".to_string()));

        let mut ws_headers = HashMap::new();
        headers.insert("User-Agent".into(), "firefox".into());
        try_default_headers_with(&mut ws_headers, "ws");
        assert_eq!(
            ws_headers.get("Sec-Fetch-Mode"),
            Some(&"websocket".to_string())
        );
    }
}
