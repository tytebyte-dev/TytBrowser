// Module: common\dice\dice_test.rs
// 1:1 Rust unit test suite corresponding to Go common\dice\dice_test.go

#[cfg(test)]
mod tests {
    use super::super::dice::*;

    #[test]
    fn test_dice_roll() {
        assert_eq!(roll(1), 0);
        let val = roll(20);
        assert!(val < 20);

        assert_eq!(roll_int63n(1), 0);
        let val64 = roll_int63n(100);
        assert!(val64 < 100);

        let _ = roll_uint16();
        let _ = roll_uint64();

        let r1 = roll_deterministic(100, 12345);
        let r2 = roll_deterministic(100, 12345);
        assert_eq!(r1, r2);

        let mut dd = DeterministicDice::new(54321);
        assert_eq!(dd.roll(1), 0);
        let d_val = dd.roll(50);
        assert!(d_val < 50);
    }
}
