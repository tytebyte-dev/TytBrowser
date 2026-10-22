// Module: common\serial\typed_message_test.rs
// 1:1 Rust unit test suite corresponding to Go common\serial\typed_message_test.go

#[cfg(test)]
mod tests {
    use super::super::typed_message::{TypedMessage, get_instance, to_typed_message};

    #[test]
    fn test_get_instance() {
        let p = get_instance("");
        assert!(p.is_err());
    }

    #[test]
    fn test_converting_nil_message() {
        let x = to_typed_message(None);
        assert!(x.is_none());
    }

    #[test]
    fn test_typed_message_serialization() {
        let msg = TypedMessage::new("xray.test.Message", b"hello world".to_vec());
        assert_eq!(msg.type_name(), "xray.test.Message");
        assert_eq!(msg.value, b"hello world");

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: TypedMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, msg);
    }
}
