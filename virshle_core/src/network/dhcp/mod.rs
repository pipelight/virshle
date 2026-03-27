pub mod fake;
pub mod kea;
pub mod lease;

// Reexports
pub use fake::{FakeDhcp, IpPool};
pub use kea::KeaDhcp;
pub use lease::Lease;
