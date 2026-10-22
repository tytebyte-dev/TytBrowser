// Module: common\common.rs
// 1:1 Rust implementation corresponding to Go common\common.go

use crate::common::errors::Result;

pub const ERR_NO_CLUE: &str = "not enough information for making a decision";

pub trait Closable: Send + Sync {
    fn close(&self) -> Result<()>;
}

pub trait Runnable: Send + Sync {
    fn start(&self) -> Result<()>;
}

pub trait HasType: Send + Sync {
    fn has_type(&self) -> &'static str;
}

pub fn must<T, E: std::fmt::Debug>(result: std::result::Result<T, E>) -> T {
    match result {
        Ok(val) => val,
        Err(err) => panic!("must assertion failed: {:?}", err),
    }
}

pub fn must2<T, E: std::fmt::Debug>(val: T, result: std::result::Result<(), E>) -> T {
    must(result);
    val
}

pub fn error2<T, E>(result: std::result::Result<T, E>) -> Option<E> {
    result.err()
}

pub fn close_if_exists<T: Closable>(obj: Option<&T>) -> Result<()> {
    if let Some(o) = obj { o.close() } else { Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyClosable;
    impl Closable for DummyClosable {
        fn close(&self) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_must_success() {
        let res: std::result::Result<i32, &str> = Ok(42);
        assert_eq!(must(res), 42);
    }

    #[test]
    #[should_panic]
    fn test_must_panic() {
        let res: std::result::Result<i32, &str> = Err("fatal error");
        must(res);
    }

    #[test]
    fn test_must2() {
        let ok: std::result::Result<(), &str> = Ok(());
        assert_eq!(must2(100, ok), 100);
    }

    #[test]
    fn test_error2() {
        let err: std::result::Result<i32, &str> = Err("test error");
        assert_eq!(error2(err), Some("test error"));
    }

    #[test]
    fn test_close_if_exists() {
        let dummy = DummyClosable;
        assert!(close_if_exists(Some(&dummy)).is_ok());
        assert!(close_if_exists::<DummyClosable>(None).is_ok());
    }
}
