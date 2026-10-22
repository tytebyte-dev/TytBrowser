// Module: common\bitmask\byte_test.rs
// 1:1 Rust unit test suite corresponding to Go common\bitmask\byte_test.go

#[cfg(test)]
mod tests {
    use super::super::byte::Byte;

    #[test]
    fn test_bitmask_byte() {
        let mut b = Byte(0);
        b.set(Byte(1));
        assert!(b.has(1), "expected {:?} to contain 1, but actually not", b);

        b.set(Byte(2));
        assert!(b.has(2), "expected {:?} to contain 2, but actually not", b);
        assert!(b.has(1), "expected {:?} to contain 1, but actually not", b);

        b.clear(Byte(1));
        assert!(b.has(2), "expected {:?} to contain 2, but actually not", b);
        assert!(
            !b.has(1),
            "expected {:?} to not contain 1, but actually did",
            b
        );

        b.toggle(Byte(2));
        assert!(
            !b.has(2),
            "expected {:?} to not contain 2, but actually did",
            b
        );
    }
}
