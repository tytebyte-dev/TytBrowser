// Module: common\mux\session_test.rs
// 1:1 Rust unit test suite corresponding to Go common\mux\session_test.go

#[cfg(test)]
mod tests {
    use crate::common::mux::mux::ClientStrategy;
    use crate::common::mux::session::{Session, SessionManager};

    #[test]
    fn test_session_manager_add() {
        let m = SessionManager::new();

        let s1 = m.allocate(&ClientStrategy::default()).expect("allocate s1");
        assert_eq!(s1.id, 1);
        assert_eq!(m.size(), 1);

        let s2 = m.allocate(&ClientStrategy::default()).expect("allocate s2");
        assert_eq!(s2.id, 2);
        assert_eq!(m.size(), 2);

        let s4 = Session::new(4);
        assert!(m.add(&s4));
        assert_eq!(s4.id, 4);
        assert_eq!(m.size(), 3);
    }

    #[test]
    fn test_session_manager_close() {
        let m = SessionManager::new();
        let s = m.allocate(&ClientStrategy::default()).expect("allocate");

        assert!(!m.close_if_no_session_and_idle(m.size(), m.count()));
        m.remove(false, s.id);
        assert!(m.close_if_no_session_and_idle(m.size(), m.count()));
    }
}
