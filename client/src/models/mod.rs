//! API response models for VolumeLeaders trade data.
//!
//! Model structs mirror server JSON field names exactly and are not
//! individually documented because the field names are the API contract.

#![allow(missing_docs)]

/// VolumeLeaders alert configuration and alert row models.
pub mod alert;
/// VolumeLeaders trade cluster and trade cluster bomb models.
pub mod cluster;
/// VolumeLeaders earnings model.
pub mod earning;
/// VolumeLeaders exhaustion score model.
pub mod exhaustion;
/// VolumeLeaders trade level model.
pub mod level;
/// VolumeLeaders institutional trade model.
pub mod trade;
/// Custom serde types for ASP.NET dates and flexible booleans.
pub mod types;
/// VolumeLeaders watchlist configuration and ticker row models.
pub mod watchlist;

pub use alert::{AlertConfig, TradeAlert, TradeClusterAlert};
pub use cluster::{TradeCluster, TradeClusterBomb};
pub use earning::Earning;
pub use exhaustion::ExhaustionScore;
pub use level::TradeLevel;
pub use trade::Trade;
pub use types::{AspNetDate, FlexBool};
pub use watchlist::{WatchListConfig, WatchListTicker};
