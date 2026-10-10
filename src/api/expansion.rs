use std::{
    collections::HashMap,
    error::Error,
    fmt,
    sync::{Arc, RwLock},
};

use pumpkin_plugin_api::ipc;

use crate::{
    ipc::protocol::{GlyphRequest, GlyphResponse, IpcRequest, IpcResponse, PROTOCOL_VERSION},
    namespaced_key::NamespacedKey,
    placeholder::PlaceholderContext,
};

pub const GLYPH_PLUGIN_NAME: &str = "Glyph";

type Resolver = dyn Fn(&PlaceholderContext) -> Option<String> + Send + Sync + 'static;

struct RegistryState {
    resolvers: HashMap<NamespacedKey, Arc<Resolver>>,
    published: bool,
    publishing: bool,
}

/// Public registry for a plugin that provides Glyph expansions.
///
/// Register local resolvers before calling `register_with_glyph()`
/// from the provider plugin's `on_load` callback.
pub struct ExpansionRegistry {
    state: RwLock<RegistryState>,
}

impl ExpansionRegistry {
    /// Creates an empty expansion registry.
    pub fn new() -> Self {
        Self {
            state: RwLock::new(RegistryState {
                resolvers: HashMap::new(),
                published: false,
                publishing: false,
            }),
        }
    }

    /// Registers a placeholder resolver.
    ///
    /// The key can use any valid namespace; it does not have to match
    /// the plugin's own name.
    ///
    /// Returns an error if the key already exists or publication has begun.
    pub fn register<F>(&self, key: NamespacedKey, resolver: F) -> Result<(), GlyphApiError>
    where
        F: Fn(&PlaceholderContext) -> Option<String> + Send + Sync + 'static,
    {
        let mut state = self
            .state
            .write()
            .unwrap_or_else(|error| error.into_inner());

        if state.published || state.publishing {
            return Err(GlyphApiError::AlreadyPublished);
        }

        if state.resolvers.contains_key(&key) {
            return Err(GlyphApiError::DuplicateKey(key));
        }

        state.resolvers.insert(key, Arc::new(resolver));

        Ok(())
    }

    /// Returns the locally registered placeholder count.
    pub fn len(&self) -> usize {
        self.state
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .resolvers
            .len()
    }

    /// Returns whether this registry has no placeholders.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Publishes all registered keys to Glyph.
    ///
    /// Call this during `on_load`, after Glyph has loaded.
    pub fn register_with_glyph(&self) -> Result<usize, GlyphApiError> {
        let keys = {
            let mut state = self
                .state
                .write()
                .unwrap_or_else(|error| error.into_inner());

            if state.published {
                return Err(GlyphApiError::AlreadyPublished);
            }

            if state.publishing {
                return Err(GlyphApiError::OperationInProgress);
            }

            if state.resolvers.is_empty() {
                return Err(GlyphApiError::NoPlaceholders);
            }

            state.publishing = true;

            let mut keys: Vec<String> = state.resolvers.keys().map(ToString::to_string).collect();

            keys.sort_unstable();
            keys
        };

        let result = (|| {
            let response = send_request(GlyphRequest::Register { keys })?;

            match response.response {
                GlyphResponse::Registered { count } => Ok(count),
                GlyphResponse::Error { message } => Err(GlyphApiError::GlyphRejected(message)),
                other => Err(GlyphApiError::UnexpectedResponse(format!("{other:?}"))),
            }
        })();

        let mut state = self
            .state
            .write()
            .unwrap_or_else(|error| error.into_inner());

        state.publishing = false;

        if result.is_ok() {
            state.published = true;
        }

        result
    }

    /// Unregisters all this provider's keys from Glyph.
    ///
    /// Local resolver functions remain registered, so the provider
    /// can publish them again if needed.
    pub fn unregister_from_glyph(&self) -> Result<usize, GlyphApiError> {
        {
            let mut state = self
                .state
                .write()
                .unwrap_or_else(|error| error.into_inner());

            if state.publishing {
                return Err(GlyphApiError::OperationInProgress);
            }

            if !state.published {
                return Ok(0);
            }

            state.publishing = true;
        }

        let result = (|| {
            let response = send_request(GlyphRequest::UnregisterAll)?;

            match response.response {
                GlyphResponse::Unregistered { count } => Ok(count),
                GlyphResponse::Error { message } => Err(GlyphApiError::GlyphRejected(message)),
                other => Err(GlyphApiError::UnexpectedResponse(format!("{other:?}"))),
            }
        })();

        let mut state = self
            .state
            .write()
            .unwrap_or_else(|error| error.into_inner());

        state.publishing = false;

        if result.is_ok() {
            state.published = false;
        }

        result
    }

    /// Handles IPC requests from Glyph.
    ///
    /// Delegate your provider plugin's `handle_ipc_message` callback
    /// to this method.
    pub fn handle_ipc_message(&self, sender: &str, message: &[u8]) -> Result<Vec<u8>, String> {
        if !sender.eq_ignore_ascii_case(GLYPH_PLUGIN_NAME) {
            return Err(format!(
                "Expansion registry only accepts resolution requests from {GLYPH_PLUGIN_NAME}"
            ));
        }

        let request: IpcRequest = serde_json::from_slice(message)
            .map_err(|error| format!("Invalid Glyph IPC request: {error}"))?;

        let response = if request.version != PROTOCOL_VERSION {
            GlyphResponse::Error {
                message: format!("Unsupported Glyph protocol version: {}", request.version),
            }
        } else {
            match request.request {
                GlyphRequest::Resolve { key, context } => match key.parse::<NamespacedKey>() {
                    Ok(key) => {
                        let resolver = {
                            let state =
                                self.state.read().unwrap_or_else(|error| error.into_inner());

                            state.resolvers.get(&key).cloned()
                        };

                        let value = resolver.and_then(|resolve| resolve(&context));

                        GlyphResponse::Resolved { value }
                    }

                    Err(error) => GlyphResponse::Error {
                        message: format!("Invalid namespaced key: {error}"),
                    },
                },

                _ => GlyphResponse::Error {
                    message: "Expansion providers only handle resolve requests.".into(),
                },
            }
        };

        serde_json::to_vec(&IpcResponse {
            version: PROTOCOL_VERSION,
            response,
        })
        .map_err(|error| format!("Could not serialize Glyph IPC response: {error}"))
    }
}

impl Default for ExpansionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Sends one versioned request to Glyph.
fn send_request(request: GlyphRequest) -> Result<IpcResponse, GlyphApiError> {
    let request = IpcRequest {
        version: PROTOCOL_VERSION,
        request,
    };

    let payload = serde_json::to_vec(&request)
        .map_err(|error| GlyphApiError::Serialization(error.to_string()))?;

    let response_bytes = match ipc::send_ipc_message(GLYPH_PLUGIN_NAME, &payload) {
        Ok(Ok(bytes)) => bytes,
        Ok(Err(message)) => {
            return Err(GlyphApiError::GlyphRejected(message));
        }
        Err(()) => return Err(GlyphApiError::IpcUnavailable),
    };

    let response: IpcResponse = serde_json::from_slice(&response_bytes)
        .map_err(|error| GlyphApiError::Serialization(error.to_string()))?;

    if response.version != PROTOCOL_VERSION {
        return Err(GlyphApiError::UnsupportedProtocolVersion(response.version));
    }

    Ok(response)
}

/// Errors from registration and communication with Glyph.
#[derive(Debug)]
pub enum GlyphApiError {
    DuplicateKey(NamespacedKey),
    NoPlaceholders,
    AlreadyPublished,
    OperationInProgress,
    IpcUnavailable,
    GlyphRejected(String),
    Serialization(String),
    UnsupportedProtocolVersion(u16),
    UnexpectedResponse(String),
}

impl fmt::Display for GlyphApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateKey(key) => {
                write!(
                    formatter,
                    "placeholder '{key}' is already registered locally"
                )
            }
            Self::NoPlaceholders => {
                formatter.write_str("cannot publish an empty expansion registry")
            }
            Self::AlreadyPublished => {
                formatter.write_str("expansion registry is already published")
            }
            Self::OperationInProgress => {
                formatter.write_str("an expansion operation is already in progress")
            }
            Self::IpcUnavailable => formatter.write_str("could not communicate with Glyph"),
            Self::GlyphRejected(message) => {
                write!(formatter, "Glyph rejected the request: {message}")
            }
            Self::Serialization(message) => {
                write!(formatter, "IPC serialization failed: {message}")
            }
            Self::UnsupportedProtocolVersion(version) => {
                write!(formatter, "unsupported Glyph protocol version: {version}")
            }
            Self::UnexpectedResponse(message) => {
                write!(formatter, "unexpected Glyph response: {message}")
            }
        }
    }
}

impl Error for GlyphApiError {}
