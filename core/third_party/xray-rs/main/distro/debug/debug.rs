// Module: main\distro\debug\debug.rs
// 1:1 Rust implementation corresponding to Go main\distro\debug\debug.go

pub const DEBUG_ENABLED: bool = cfg!(debug_assertions);
pub const IS_DEBUG: bool = DEBUG_ENABLED;

pub fn is_debug_build() -> bool {
    DEBUG_ENABLED
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distro_debug() {
        let _ = is_debug_build();
    }
}
