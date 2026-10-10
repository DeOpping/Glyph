use pumpkin_plugin_api::{Context, Plugin, PluginMetadata};
use tracing::info;

use crate::PlaceholderApi;

pub(crate) struct GlyphPlugin {
    placeholders: PlaceholderApi<()>
}

impl Plugin for GlyphPlugin {
    fn new() -> Self {
        Self {
            placeholders: PlaceholderApi::new()
        }
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "Glyph".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["DeOpping".into()],
            description: "A placeholder API for Pupmkin.".into(),
            dependencies: vec![],
            permissions: vec![]
        }
    }

    fn on_load(&self, _context: Context) -> pumpkin_plugin_api::Result<()> {
        info!(
            "Glyph v{} loaded with {} registered placeholders.",
            env!("CARGO_PKG_VERSION"),
            self.placeholders.len()
        );

        Ok(())
    }

    fn on_unload(&self, _context: Context) -> pumpkin_plugin_api::Result<()> {
        info!("Glyph unloaded.");

        Ok(())
    }
}