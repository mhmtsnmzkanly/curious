//! Public snapshots share the engine checkpoint representation.
//! Resources retain their kind, and lineage/environment are visible to observers.
pub use super::checkpoint::{Individual as EntitySnapshot, State as WorldSnapshot};
