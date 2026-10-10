use std::{
    fs,
    path::PathBuf
};

use pumpkin_plugin_api::{
    ipc::{IpcMessage, PluginId},
    permissions,
    Context,
    Plugin,
    PluginMetadata
};

use tracing::info;

use crate::{
    expansion::remote_registry::RemoteExpansionRegistry,
    namespaced_key::NamespacedKey,
    placeholder::{PlaceholderApi, PlaceholderContext}
};

mod ipc;

pub(crate) struct GlyphPlugin {
    placeholders: PlaceholderApi<PlaceholderContext>,
    remote_expansions: RemoteExpansionRegistry
}

impl Plugin for GlyphPlugin {
    fn new() -> Self {
        let placeholders = PlaceholderApi::<PlaceholderContext>::new();

        // Register a built-in version placeholder for Glyph
        // %glyph:version%
        let version_key = NamespacedKey::new("glyph", "version")
            .expect("built-in namespaced key must be valid");

        placeholders
            .register(version_key, |_| {
                Some(env!("CARGO_PKG_VERSION").to_owned())
            })
            .expect("built-in placeholder must not be duplicated");
        
        Self {
            placeholders,
            remote_expansions: RemoteExpansionRegistry::default()
        }
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "Glyph".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["DeOpping".into()],
            description: "A placeholder API for Pupmkin.".into(),
            dependencies: vec![],
            permissions: vec![permissions::FS_WRITE_DATA.into()]
        }
    }

    fn on_load(&self, context: Context) -> pumpkin_plugin_api::Result<()> {
        let data_folder = PathBuf::from(context.get_data_folder());
        let expansions_folder = data_folder.join("expansions");

        fs::create_dir_all(&expansions_folder)
            .map_err(|error| {
                format!(
                    "Failed to create expansion directory '{}': {error}",
                    expansions_folder.display()
                )
            })?;

        info!(
            "Glyph v{} loaded.",
            env!("CARGO_PKG_VERSION")
        );
        
        info!(
            "Expansion directory prepared: {}",
            expansions_folder.display()
        );

        Ok(())
    }

    fn on_unload(&self, _context: Context) -> pumpkin_plugin_api::Result<()> {
        info!("Glyph unloaded.");

        Ok(())
    }

    fn handle_ipc_message(
        &self,
        sender: PluginId,
        message: IpcMessage
    ) -> pumpkin_plugin_api::Result<IpcMessage> {
        ipc::handle_message(self, sender, message)
    }
}