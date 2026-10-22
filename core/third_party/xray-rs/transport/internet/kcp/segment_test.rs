// Module: transport\internet\kcp\segment_test.rs
// 1:1 Rust unit test suite corresponding to Go transport\internet\kcp\segment_test.go

#[cfg(test)]
mod tests {
    use super::super::segment::{
        AckSegment, COMMAND_TERMINATE, CmdOnlySegment, DataSegment, Segment, read_segment,
    };

    #[test]
    fn test_kcp_segment_encode_decode() {
        let mut ack = AckSegment::new(1001, 128);
        ack.receiving_window = 256;
        ack.receiving_next = 100;
        ack.timestamp = 54321;
        ack.put_number(100);
        ack.put_number(101);
        ack.put_number(105);

        let mut buf = [0u8; 64];
        let n = ack.serialize(&mut buf).unwrap();

        let (seg, remaining) = read_segment(&buf[..n]).unwrap();
        assert!(remaining.is_empty());
        match seg {
            Segment::Ack(decoded_ack) => {
                assert_eq!(decoded_ack.conv, 1001);
                assert_eq!(decoded_ack.receiving_window, 256);
                assert_eq!(decoded_ack.receiving_next, 100);
                assert_eq!(decoded_ack.timestamp, 54321);
                assert_eq!(decoded_ack.number_list, vec![100, 101, 105]);
            }
            _ => panic!("Expected Ack segment"),
        }

        let mut data = DataSegment::new(1001, 42, b"kcp-reliable-data".to_vec());
        data.timestamp = 12345;
        let mut data_buf = [0u8; 128];
        let dn = data.serialize(&mut data_buf).unwrap();
        let (seg_data, remaining_data) = read_segment(&data_buf[..dn]).unwrap();
        assert!(remaining_data.is_empty());
        match seg_data {
            Segment::Data(decoded_data) => {
                assert_eq!(decoded_data.conv, 1001);
                assert_eq!(decoded_data.number, 42);
                assert_eq!(decoded_data.timestamp, 12345);
                assert_eq!(decoded_data.payload, b"kcp-reliable-data");
            }
            _ => panic!("Expected Data segment"),
        }

        let mut cmd = CmdOnlySegment::new(1001, COMMAND_TERMINATE);
        cmd.sending_next = 99;
        cmd.receiving_next = 100;
        let mut cmd_buf = [0u8; 32];
        let cn = cmd.serialize(&mut cmd_buf).unwrap();
        let (seg_cmd, remaining_cmd) = read_segment(&cmd_buf[..cn]).unwrap();
        assert!(remaining_cmd.is_empty());
        match seg_cmd {
            Segment::CmdOnly(decoded_cmd) => {
                assert_eq!(decoded_cmd.conv, 1001);
                assert_eq!(decoded_cmd.cmd, COMMAND_TERMINATE);
                assert_eq!(decoded_cmd.sending_next, 99);
                assert_eq!(decoded_cmd.receiving_next, 100);
            }
            _ => panic!("Expected CmdOnly segment"),
        }
    }
}
