// Module: common\reflect\marshal_test.rs
// 1:1 Rust unit test suite corresponding to Go common\reflect\marshal_test.go

#[cfg(test)]
mod tests {
    use super::super::marshal::{from_json, json_marshal_without_escape, marshal_to_json, to_json};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct UserInfo {
        email: String,
        level: u32,
    }

    #[test]
    fn test_reflect_json_roundtrip() {
        let original = vec!["xray".to_string(), "rust".to_string()];
        let json = to_json(&original).unwrap();
        let parsed: Vec<String> = from_json(&json).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn test_marshal_to_json_and_without_escape() {
        let user = UserInfo {
            email: "love@v2ray.com".to_string(),
            level: 0,
        };

        let (json_str, ok) = marshal_to_json(&user, false);
        assert!(ok);
        assert!(json_str.contains("love@v2ray.com"));
        assert!(!json_str.contains("_TypedMessage_"));

        let (json_with_type, ok) = marshal_to_json(&user, true);
        assert!(ok);
        assert!(json_with_type.contains("_TypedMessage_"));

        let bytes = json_marshal_without_escape(&user).unwrap();
        assert!(bytes.ends_with(b"\n"));
    }
}
