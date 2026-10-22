// Module: core\context_test.rs
// 1:1 Rust unit test suite corresponding to Go core\context_test.go

#[cfg(test)]
mod tests {
    use crate::core::Instance;
    use crate::core::context::{
        CoreContext, from_context, must_from_context, to_background_detached_context, to_context,
    };
    use crate::infra::conf::Config;
    use std::sync::Arc;

    #[test]
    fn test_core_instance_from_config() {
        let cfg = Config::default();
        let inst = Instance::from_config(cfg);
        assert!(inst.is_ok());
    }

    #[test]
    fn test_context_empty() {
        let ctx = CoreContext::new();
        assert!(ctx.get_instance().is_none());
        assert!(ctx.must_get_instance().is_err());
        assert!(from_context(&ctx).is_none());
        assert!(must_from_context(&ctx).is_err());
        assert!(to_background_detached_context(&ctx).is_err());
    }

    #[test]
    fn test_context_with_instance() {
        let cfg = Config::default();
        let inst = Arc::new(Instance::from_config(cfg).unwrap());
        let ctx = CoreContext::with_instance(inst.clone());

        assert!(ctx.get_instance().is_some());
        assert!(ctx.must_get_instance().is_ok());
        assert!(from_context(&ctx).is_some());
        assert!(must_from_context(&ctx).is_ok());

        let detached = to_background_detached_context(&ctx);
        assert!(detached.is_ok());
        assert!(detached.unwrap().get_instance().is_some());

        let mut ctx2 = CoreContext::new();
        to_context(&mut ctx2, inst);
        assert!(ctx2.get_instance().is_some());
    }
}
