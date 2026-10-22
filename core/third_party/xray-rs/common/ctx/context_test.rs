// Module: common\ctx\context_test.rs

#[cfg(test)]
mod tests {
    use super::super::context::*;

    #[test]
    fn test_context_id() {
        let ctx = Context::new();
        assert_eq!(id_from_context(&ctx), 0);

        let ctx2 = context_with_id(&ctx, 42);
        assert_eq!(id_from_context(&ctx2), 42);

        ctx.with_id(100);
        assert_eq!(id_from_context(&ctx), 100);
    }
}
