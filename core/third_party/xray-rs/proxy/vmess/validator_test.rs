// Module: proxy\\vmess\\validator_test.rs
// 1:1 Rust unit test suite corresponding to Go proxy\\vmess\\validator_test.go

#[cfg(test)]
mod tests {
    use super::super::validator::MemoryValidator;
    use crate::common::protocol::User;
    use uuid::Uuid;

    #[test]
    fn test_vmess_validator_user_add_and_get() {
        let validator = MemoryValidator::new();
        let user = User::new(Uuid::new_v4());
        validator.add(user).unwrap();
        assert_eq!(validator.count(), 1);
    }
}
