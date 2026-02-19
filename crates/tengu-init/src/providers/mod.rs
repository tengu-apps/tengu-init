//! Cloud provider implementations

pub mod baremetal;
pub mod hetzner;

pub use baremetal::Baremetal;
pub use hetzner::Hetzner;
