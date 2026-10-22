// Module: testing\scenarios\main_test.rs
#[cfg(test)]
mod tests {
    use crate::main::distro::DISTRO;
    use crate::main::distro::IS_DEBUG;

    #[test]
    fn test_scenario_distro_and_debug() {
        assert_eq!(DISTRO, "all");
        let _ = IS_DEBUG;
    }
}
