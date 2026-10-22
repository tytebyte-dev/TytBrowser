// Module: app\observatory\burst\healthping_result_test.rs
// 1:1 Rust unit test suite corresponding to Go app\observatory\burst\healthping_result_test.go

#[cfg(test)]
mod tests {
    use super::super::burst::RTT_FAILED;
    use super::super::healthping_result::{HealthPingStats, new_health_ping_result};
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_health_ping_results() {
        let rtts: Vec<u64> = vec![60, 140, 60, 140, 60, 60, 140, 60, 140];
        let mut hr = new_health_ping_result(4, Duration::from_secs(3600));
        for rtt in rtts {
            hr.put(Duration::from_nanos(rtt));
        }

        let expected = HealthPingStats {
            all: 4,
            fail: 0,
            deviation: Duration::from_nanos(40),
            average: Duration::from_nanos(100),
            max: Duration::from_nanos(140),
            min: Duration::from_nanos(60),
        };

        let actual = hr.get();
        assert_eq!(actual, expected);

        hr.put(RTT_FAILED);
        hr.put(RTT_FAILED);

        let mut expected2 = expected;
        expected2.fail = 2;
        let actual2 = hr.get();
        assert_eq!(actual2, expected2);

        hr.put(RTT_FAILED);
        hr.put(RTT_FAILED);

        let expected_all_fail = HealthPingStats {
            all: 4,
            fail: 4,
            deviation: Duration::ZERO,
            average: Duration::ZERO,
            max: Duration::ZERO,
            min: Duration::ZERO,
        };
        let actual_all_fail = hr.get();
        assert_eq!(actual_all_fail, expected_all_fail);
    }

    #[test]
    fn test_health_ping_results_ignore_outdated() {
        let rtts: Vec<u64> = vec![60, 140, 60, 140];
        let mut hr = new_health_ping_result(4, Duration::from_millis(15));
        for (i, rtt) in rtts.into_iter().enumerate() {
            if i == 2 {
                // wait for previous 2 to become outdated
                sleep(Duration::from_millis(20));
            }
            hr.put(Duration::from_nanos(rtt));
        }

        let actual = hr.get();
        let expected = HealthPingStats {
            all: 2,
            fail: 0,
            deviation: Duration::from_nanos(40),
            average: Duration::from_nanos(100),
            max: Duration::from_nanos(140),
            min: Duration::from_nanos(60),
        };
        assert_eq!(actual, expected);

        // wait for all to become outdated
        sleep(Duration::from_millis(20));
        let expected_outdated = HealthPingStats {
            all: 0,
            fail: 0,
            deviation: Duration::ZERO,
            average: Duration::ZERO,
            max: Duration::ZERO,
            min: Duration::ZERO,
        };
        assert_eq!(hr.get(), expected_outdated);

        hr.put(Duration::from_nanos(60));
        let expected_single = HealthPingStats {
            all: 1,
            fail: 0,
            deviation: Duration::from_nanos(30), // 1 sample, std = avg / 2
            average: Duration::from_nanos(60),
            max: Duration::from_nanos(60),
            min: Duration::from_nanos(60),
        };
        assert_eq!(hr.get(), expected_single);
    }
}
