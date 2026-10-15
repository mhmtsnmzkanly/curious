//! Public snapshots share the engine checkpoint representation.
//! Resources retain their kind, and lineage/environment are visible to observers.
pub use super::checkpoint::{State as WorldSnapshot,Individual as EntitySnapshot};
