pub mod ipc;
pub mod namespaced_key;
pub mod placeholder;

mod expansion;
mod plugin;

pub use namespaced_key::{NamespacedKey, NamespacedKeyError};
pub use placeholder::{PlaceholderApi, RegistrationError};

pumpkin_plugin_api::register_plugin!(plugin::GlyphPlugin);
