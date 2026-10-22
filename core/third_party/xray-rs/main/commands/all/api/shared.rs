// Module: main\commands\all\api\shared.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\api\shared.go

pub struct ApiClientConfig {
    pub server: String,
    pub timeout: u64,
}

impl ApiClientConfig {
    pub fn parse(args: &[&str]) -> Self {
        let mut server = "127.0.0.1:10085".to_string();
        for arg in args {
            if let Some(s) = arg.strip_prefix("-s=") {
                server = s.to_string();
            }
        }
        Self { server, timeout: 5 }
    }
}
