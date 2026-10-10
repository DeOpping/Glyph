use serde::{Serialize, Deserialize};

use crate::placeholder::PlaceholderContext;

/// Version of Glyph's cross-plugin IPC protocol.
pub const PROTOCOL_VERSION: u16 = 1;

/// Request sent to Glyph or an expansion provider.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GlyphRequest {
    /// Register multiple placeholder keys for the sending plugin.
    Register {
        keys: Vec<String>
    },

    /// Unregister specific keys owned by the sending plugin.
    Unregister {
        keys: Vec<String>
    },

    /// Unregister all keys owned by the sending plugin.
    UnregisterAll,

    /// Parse an input string through Glyph.
    Parse {
        input: String,
        context: PlaceholderContext
    },

    /// Resolve an individual key through an expansion provider.
    Resolve {
        key: String,
        context: PlaceholderContext
    }
}

/// An IPC request envelope.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IpcRequest {
    pub version: u16,
    pub request: GlyphRequest
}

/// Response returned by Glyph or an expansion provider.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GlyphResponse {
    Registered {
        count: usize
    },

    Unregistered {
        count: usize
    },

    Parsed {
        output: String
    },

    Resolved {
        value: Option<String>
    },

    Error {
        message: String
    }
}

/// An IPC response envelope.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IpcResponse {
    pub version: u16,
    pub response: GlyphResponse
}