// Module: transport\internet\headers\http\config.rs
// 1:1 Rust implementation corresponding to Go transport\internet\headers\http\config.go

use rand::Rng;

pub use super::config_pb::{
    Config, Header, Method, RequestConfig, ResponseConfig, Status, Version,
};

fn pick_string(arr: &[String]) -> String {
    match arr.len() {
        0 => String::new(),
        1 => arr[0].clone(),
        n => {
            let mut rng = rand::thread_rng();
            let idx = rng.gen_range(0..n);
            arr[idx].clone()
        }
    }
}

impl RequestConfig {
    pub fn new() -> Self {
        Self {
            version: Some(Version {
                value: "1.1".into(),
            }),
            method: Some(Method {
                value: "GET".into(),
            }),
            uri: vec!["/".into()],
            header: Vec::new(),
        }
    }

    pub fn format_request(&self, host: &str) -> String {
        let uri = self.pick_uri();
        let mut out = format!(
            "{} {} {}\r\n",
            self.get_method_value(),
            uri,
            self.get_full_version()
        );
        out.push_str(&format!("Host: {}\r\n", host));
        for h in self.pick_headers() {
            if !h.to_lowercase().starts_with("host:") {
                out.push_str(&h);
                out.push_str("\r\n");
            }
        }
        out.push_str("\r\n");
        out
    }

    pub fn pick_uri(&self) -> String {
        let uri = pick_string(&self.uri);
        if uri.is_empty() { "/".into() } else { uri }
    }

    pub fn pick_headers(&self) -> Vec<String> {
        let mut headers = Vec::with_capacity(self.header.len());
        for h in &self.header {
            let val = pick_string(&h.value);
            headers.push(format!("{}: {}", h.name, val));
        }
        headers
    }

    pub fn get_version_value(&self) -> &str {
        self.version
            .as_ref()
            .map(|v| v.value.as_str())
            .unwrap_or("1.1")
    }

    pub fn get_method_value(&self) -> &str {
        self.method
            .as_ref()
            .map(|m| m.value.as_str())
            .unwrap_or("GET")
    }

    pub fn get_full_version(&self) -> String {
        format!("HTTP/{}", self.get_version_value())
    }
}

impl ResponseConfig {
    pub fn new() -> Self {
        Self {
            version: Some(Version {
                value: "1.1".into(),
            }),
            status: Some(Status {
                code: "200".into(),
                reason: "OK".into(),
            }),
            header: Vec::new(),
        }
    }

    pub fn format_response(&self) -> String {
        let mut out = format!(
            "{} {} {}\r\n",
            self.get_full_version(),
            self.get_status_code(),
            self.get_status_reason()
        );
        for h in self.pick_headers() {
            out.push_str(&h);
            out.push_str("\r\n");
        }
        out.push_str("\r\n");
        out
    }

    pub fn has_header(&self, header_name: &str) -> bool {
        self.header
            .iter()
            .any(|h| h.name.eq_ignore_ascii_case(header_name))
    }

    pub fn pick_headers(&self) -> Vec<String> {
        let mut headers = Vec::with_capacity(self.header.len());
        for h in &self.header {
            let val = pick_string(&h.value);
            headers.push(format!("{}: {}", h.name, val));
        }
        headers
    }

    pub fn get_version_value(&self) -> &str {
        self.version
            .as_ref()
            .map(|v| v.value.as_str())
            .unwrap_or("1.1")
    }

    pub fn get_full_version(&self) -> String {
        format!("HTTP/{}", self.get_version_value())
    }

    pub fn get_status_code(&self) -> &str {
        self.status
            .as_ref()
            .map(|s| s.code.as_str())
            .unwrap_or("200")
    }

    pub fn get_status_reason(&self) -> &str {
        self.status
            .as_ref()
            .map(|s| s.reason.as_str())
            .unwrap_or("OK")
    }
}
