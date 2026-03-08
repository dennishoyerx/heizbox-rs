//! Re-export from `heizbox-core` so existing callers in heizbox-infra
//! and heizbox-esp32 that import from this path still compile.
pub use heizbox_core::network::ExponentialBackoff;
