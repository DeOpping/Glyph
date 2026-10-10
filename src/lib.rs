pub mod api;
pub mod ipc;
pub mod namespaced_key;
pub mod placeholder;

#[cfg(feature = "plugin")]
mod expansion;

#[cfg(feature = "plugin")]
mod plugin;

pub use namespaced_key::{NamespacedKey, NamespacedKeyError};

pub use placeholder::{PlaceholderApi, PlaceholderContext, PlayerContext, RegistrationError};

#[cfg(feature = "plugin")]
pumpkin_plugin_api::register_plugin!(plugin::GlyphPlugin);
