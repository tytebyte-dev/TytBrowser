#[cfg(test)]
mod tests {
    use crate::common::bitmask::ByteMask;
    use crate::common::dice::{roll, roll_deterministic, roll_u16, roll_u64};

    #[test]
    fn test_dice_roll_ranges() {
        for _ in 0..100 {
            let r = roll(10);
            assert!(r < 10);
        }
        let _ = roll_u16();
        let _ = roll_u64();
        let det1 = roll_deterministic(100, 12345);
        let det2 = roll_deterministic(100, 12345);
        assert_eq!(det1, det2);
    }

    #[test]
    fn test_bitmask_operations() {
        let mut mask = ByteMask::new(0b0000_0001);
        let flag_b = ByteMask::new(0b0000_0010);

        assert!(mask.has(ByteMask::new(0b0000_0001)));
        assert!(!mask.has(flag_b));

        mask.set(flag_b);
        assert_eq!(mask.value(), 0b0000_0011);
        assert!(mask.has(flag_b));

        mask.clear(flag_b);
        assert_eq!(mask.value(), 0b0000_0001);
        assert!(!mask.has(flag_b));

        mask.toggle(flag_b);
        assert_eq!(mask.value(), 0b0000_0011);
        mask.toggle(flag_b);
        assert_eq!(mask.value(), 0b0000_0001);
    }
}
