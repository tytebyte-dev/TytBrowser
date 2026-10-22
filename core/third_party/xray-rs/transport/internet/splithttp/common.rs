// Module: transport\internet\splithttp\common.rs
// 1:1 Rust implementation corresponding to Go transport\internet\splithttp\common.go

pub const PLACEMENT_QUERY_IN_HEADER: &str = "queryInHeader";
pub const PLACEMENT_COOKIE: &str = "cookie";
pub const PLACEMENT_HEADER: &str = "header";
pub const PLACEMENT_QUERY: &str = "query";
pub const PLACEMENT_PATH: &str = "path";
pub const PLACEMENT_BODY: &str = "body";
pub const PLACEMENT_AUTO: &str = "auto";

pub const DEFAULT_MAX_CONCURRENT_UPLOADS: usize = 8;
pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;
pub const HEADER_SEQUENCE_ID: &str = "X-Seq-ID";
pub const HEADER_PADDING: &str = "X-Padding";
