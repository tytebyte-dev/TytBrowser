// Module: common\protocol\bittorrent\bittorrent.rs
// 1:1 Rust implementation corresponding to Go common\protocol\bittorrent\bittorrent.go

use crate::common::errors::{Error, Result};

pub struct BittorrentSniffer;

impl BittorrentSniffer {
    pub fn sniff(bytes: &[u8]) -> Result<String> {
        if bytes.len() < 20 {
            return Err(Error::Protocol("Payload too short for BitTorrent".into()));
        }

        if bytes[0] == 19 && &bytes[1..20] == b"BitTorrent protocol" {
            return Ok("bittorrent".into());
        }

        // uTP check
        let type_and_ver = bytes[0];
        let ver = type_and_ver & 0x0F;
        let p_type = type_and_ver >> 4;
        if ver == 1 && p_type <= 4 {
            let extension = bytes[1];
            if extension == 0 || extension == 1 {
                return Ok("utp".into());
            }
        }

        Err(Error::Protocol("Not a BitTorrent packet".into()))
    }
}
