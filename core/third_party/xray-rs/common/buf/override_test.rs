// Module: common\buf\override_test.rs
// 1:1 Rust unit test suite for EndpointOverrideReader and EndpointOverrideWriter

#[cfg(test)]
mod tests {
    use crate::common::buf::buffer::Buffer;
    use crate::common::buf::io::{Reader, Writer};
    use crate::common::buf::multi_buffer::MultiBuffer;
    use crate::common::buf::override_::{EndpointOverrideReader, EndpointOverrideWriter};
    use crate::common::errors::Result;
    use crate::common::net::{Address, Destination};
    use async_trait::async_trait;
    use std::net::{IpAddr, Ipv4Addr};

    struct MockReader {
        mb: Option<MultiBuffer>,
    }

    #[async_trait::async_trait]
    impl Reader for MockReader {
        async fn read_multi_buffer(&mut self) -> Result<MultiBuffer> {
            Ok(self.mb.take().unwrap_or_default())
        }
    }

    struct MockWriter {
        written: Vec<MultiBuffer>,
    }

    #[async_trait]
    impl Writer for MockWriter {
        async fn write_multi_buffer(&mut self, mb: MultiBuffer) -> Result<()> {
            self.written.push(mb);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_endpoint_override_reader_and_writer() {
        let orig_addr = Address::from(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)));
        let dest_addr = Address::from(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));

        let mut b = Buffer::from_bytes(b"udp payload");
        b.udp = Some(Destination::udp(orig_addr.clone(), 53));
        let mut mb = MultiBuffer::new();
        mb.push(b);

        let mock_r = MockReader { mb: Some(mb) };
        let mut override_r =
            EndpointOverrideReader::new(mock_r, dest_addr.clone(), orig_addr.clone());

        let res_mb = override_r.read_multi_buffer().await.unwrap();
        assert_eq!(res_mb.buffers()[0].udp.as_ref().unwrap().address, dest_addr);

        let mock_w = MockWriter {
            written: Vec::new(),
        };
        let mut override_w =
            EndpointOverrideWriter::new(mock_w, dest_addr.clone(), orig_addr.clone());

        override_w.write_multi_buffer(res_mb).await.unwrap();
        assert_eq!(
            override_w.writer.written[0].buffers()[0]
                .udp
                .as_ref()
                .unwrap()
                .address,
            orig_addr
        );
    }
}
