mod api;
pub(crate) mod parser;
mod context;

pub use api::{PlaceholderApi, RegistrationError};
pub use context::{PlaceholderContext, PlayerContext};