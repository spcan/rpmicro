//! Internal memory module.

pub mod bus;
pub mod cache;
pub mod perf;

pub use bus::{BusControl, BusPriority, MasterBus};
pub use cache::{/*CacheControl, */ CachePerformance};
pub use perf::{BusEvent, BusPerformance, EventKind, GlobalPerformance};
