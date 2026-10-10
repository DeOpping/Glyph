use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Information available while resolving placeholders.
///
/// This type is serialized across plugin boundaries.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlaceholderContext {
    #[serde(default)]
    pub player: Option<PlayerContext>,

    #[serde(default)]
    pub values: BTreeMap<String, serde_json::Value>
}

/// Basic information about the requesting player.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerContext {
    pub uuid: String,
    pub name: String,

    #[serde(default)]
    pub locale: Option<String>
}