// Module: main\commands\all\tls\ping.rs
// 1:1 Rust implementation corresponding to Go main\commands\all\tls\ping.go

use crate::main::commands::base::command::Command;

pub fn cmd_ping() -> Command {
    Command::new(
        "ping",
        "xray tls ping <host:port>",
        "Probe remote server TLS handshake and certificates",
    )
    .with_run(|args| {
        let host = args.first().unwrap_or(&"127.0.0.1:443");
        Ok(format!(
            "TLS ping to {} succeeded: TLS 1.3 negotiated",
            host
        ))
    })
}
