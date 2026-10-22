// Module: common\protocol\address_test.rs
// 1:1 Rust unit test suite corresponding to Go common\protocol\address_test.go

#[cfg(test)]
mod tests {
    use super::super::address::AddressParser;
    use crate::common::net::Address;
    use std::io::Cursor;
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn test_address_parser_ipv4() {
        let parser = AddressParser::new(false);
        let mut buf = Vec::new();
        let addr = Address::Ipv4(Ipv4Addr::new(192, 168, 1, 100));
        let port = 8080u16;

        parser
            .write_address_port(&mut buf, &addr, port)
            .await
            .unwrap();

        let mut reader = Cursor::new(buf);
        let (decoded_addr, decoded_port) = parser.read_address_port(&mut reader).await.unwrap();
        assert_eq!(decoded_addr, addr);
        assert_eq!(decoded_port, port);
    }

    #[tokio::test]
    async fn test_address_parser_domain() {
        let parser = AddressParser::new(true);
        let mut buf = Vec::new();
        let addr = Address::Domain("www.example.org".into());
        let port = 443u16;

        parser
            .write_address_port(&mut buf, &addr, port)
            .await
            .unwrap();

        let mut reader = Cursor::new(buf);
        let (decoded_addr, decoded_port) = parser.read_address_port(&mut reader).await.unwrap();
        assert_eq!(decoded_addr, addr);
        assert_eq!(decoded_port, port);
    }
}
