use pumpkin_plugin_api::ipc::{self as pumpkin_ipc, IpcMessage, PluginId};

use tracing::warn;

use super::GlyphPlugin;

use crate::{
    expansion::remote_registry::RemoteExpansionRegistry,
    ipc::protocol::{GlyphRequest, GlyphResponse, IpcRequest, IpcResponse, PROTOCOL_VERSION},
    namespaced_key::NamespacedKey,
    placeholder::{PlaceholderApi, PlaceholderContext, parser},
};

/// Parses text using the shared local and remote registries.
///
/// This is shared by the command handler and the IPC handler.
pub(crate) fn parse_with_registries(
    placeholders: &PlaceholderApi<PlaceholderContext>,
    remote_expansions: &RemoteExpansionRegistry,
    input: &str,
    context: &PlaceholderContext,
) -> String {
    parser::parse(input, |key| {
        placeholders
            .resolve(key, context)
            .or_else(|| resolve_remote(remote_expansions, key, context))
    })
}

/// Resolves a registered placeholder through its remote provider.
fn resolve_remote(
    remote_expansions: &RemoteExpansionRegistry,
    key: &NamespacedKey,
    context: &PlaceholderContext,
) -> Option<String> {
    let owner = remote_expansions.owner(key)?;

    let request = IpcRequest {
        version: PROTOCOL_VERSION,
        request: GlyphRequest::Resolve {
            key: key.to_string(),
            context: context.clone(),
        },
    };

    let payload = match serde_json::to_vec(&request) {
        Ok(payload) => payload,
        Err(error) => {
            warn!(%error, "Could not serialize Glyph resolve request");
            return None;
        }
    };

    let response_bytes = match pumpkin_ipc::send_ipc_message(&owner, &payload) {
        Ok(Ok(bytes)) => bytes,

        Ok(Err(error)) => {
            warn!(
                provider = %owner,
                key = %key,
                %error,
                "Expansion provider rejected resolve request"
            );

            return None;
        }

        Err(()) => {
            warn!(
                provider = %owner,
                key = %key,
                "IPC call to expansion provider failed"
            );

            return None;
        }
    };

    let response: IpcResponse = match serde_json::from_slice(&response_bytes) {
        Ok(response) => response,
        Err(error) => {
            warn!(
                provider = %owner,
                %error,
                "Invalid response from expansion provider"
            );

            return None;
        }
    };

    if response.version != PROTOCOL_VERSION {
        warn!(
            provider = %owner,
            version = response.version,
            "Unsupported expansion provider protocol version"
        );

        return None;
    }

    match response.response {
        GlyphResponse::Resolved { value } => value,

        GlyphResponse::Error { message } => {
            warn!(
                provider = %owner,
                key = %key,
                %message,
                "Expansion resolution failed"
            );

            None
        }

        _ => {
            warn!(
                provider = %owner,
                key = %key,
                "Unexpected response to resolve request"
            );

            None
        }
    }
}

impl GlyphPlugin {
    /// Processes requests originating from other plugins.
    fn handle_request(&self, sender: &str, request: GlyphRequest) -> GlyphResponse {
        match request {
            GlyphRequest::Register { keys } => {
                let parsed_keys: Result<Vec<NamespacedKey>, _> =
                    keys.iter().map(|key| key.parse()).collect();

                let parsed_keys = match parsed_keys {
                    Ok(keys) => keys,
                    Err(error) => {
                        return GlyphResponse::Error {
                            message: format!("invalid namespaced key: {error}"),
                        };
                    }
                };

                // Remote providers cannot claim a local key.
                for key in &parsed_keys {
                    if self.placeholders.contains(key) {
                        return GlyphResponse::Error {
                            message: format!("placeholder '{key}' is already registered locally"),
                        };
                    }
                }

                match self.remote_expansions.register_many(sender, parsed_keys) {
                    Ok(count) => GlyphResponse::Registered { count },
                    Err(message) => GlyphResponse::Error { message },
                }
            }

            GlyphRequest::Unregister { keys } => {
                let parsed_keys: Result<Vec<NamespacedKey>, _> =
                    keys.iter().map(|key| key.parse()).collect();

                let parsed_keys = match parsed_keys {
                    Ok(keys) => keys,
                    Err(error) => {
                        return GlyphResponse::Error {
                            message: format!("invalid namespaced key: {error}"),
                        };
                    }
                };

                let count = self.remote_expansions.unregister_many(sender, &parsed_keys);

                GlyphResponse::Unregistered { count }
            }

            GlyphRequest::UnregisterAll => {
                let count = self.remote_expansions.unregister_all(sender);

                GlyphResponse::Unregistered { count }
            }

            GlyphRequest::Parse { input, context } => GlyphResponse::Parsed {
                output: parse_with_registries(
                    &self.placeholders,
                    &self.remote_expansions,
                    &input,
                    &context,
                ),
            },

            GlyphRequest::Resolve { .. } => GlyphResponse::Error {
                message: "Resolve requests must be sent to expansion providers".into(),
            },
        }
    }
}

/// IPC entry point
pub(super) fn handle_message(
    plugin: &GlyphPlugin,
    sender: PluginId,
    message: IpcMessage,
) -> pumpkin_plugin_api::Result<IpcMessage> {
    let request: IpcRequest = serde_json::from_slice(&message)
        .map_err(|error| format!("Invalid Glyph IPC request: {error}"))?;

    let response = if request.version != PROTOCOL_VERSION {
        GlyphResponse::Error {
            message: format!("Unsupported Glyph protocol version: {}", request.version),
        }
    } else {
        plugin.handle_request(&sender, request.request)
    };

    serde_json::to_vec(&IpcResponse {
        version: PROTOCOL_VERSION,
        response,
    })
    .map_err(|error| format!("Could not serialize Glyph IPC response: {error}"))
}
