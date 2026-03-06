#[cfg(test)]
mod network_tests {
    use heizbox_core::error::NetworkError;

    #[test]
    fn network_error_display() {
        let e = NetworkError::Timeout;
        assert!(!e.to_string().is_empty());
    }

    #[test]
    fn reconnect_failed_display() {
        let e = NetworkError::ReconnectFailed;
        assert!(e.to_string().contains("retries"));
    }
}
