//! Trade-shaped row family labels used for field metadata and defaults.

/// Trade-shaped row families that share output field metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TradeRecordKind {
    /// Individual institutional trade rows.
    Trade,
    /// Aggregated trade cluster rows.
    Cluster,
    /// Trade price-level rows.
    Level,
    /// Trade cluster bomb rows.
    ClusterBomb,
}
