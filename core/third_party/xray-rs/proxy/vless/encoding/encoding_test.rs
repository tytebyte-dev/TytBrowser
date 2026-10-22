// Module: proxy\vless\encoding\encoding_test.rs
// 1:1 Rust unit test suite corresponding to Go proxy\vless\encoding\encoding_test.go

#[cfg(test)]
mod tests {
    use super::super::addons::Addons;

    #[test]
    fn test_vless_addons() {
        let addons = Addons::new("xtls-rprx-vision");
        assert_eq!(addons.flow, "xtls-rprx-vision");
    }
}
