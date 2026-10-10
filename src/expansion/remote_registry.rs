use std::{
    collections::{HashMap, HashSet},
    sync::RwLock
};

use crate::namespaced_key::NamespacedKey;

/// Tracks which plugin owns each remotely registered placeholder.
#[derive(Default)]
pub(crate) struct RemoteExpansionRegistry {
    owners: RwLock<HashMap<NamespacedKey, String>>
}

impl RemoteExpansionRegistry {
    /// Registers keys on behalf of one plugin.
    ///
    /// Registration is atomic: either every key is registered,
    /// or none are.
    pub fn register_many(
        &self,
        owner: &str,
        keys: Vec<NamespacedKey>
    ) -> Result<usize, String> {
        if keys.is_empty() {
            return Err("at least one placeholder key is required".into());
        }

        let mut owners = self
            .owners
            .write()
            .unwrap_or_else(|error| error.into_inner());

        let mut seen = HashSet::new();

        for key in &keys {
            if !seen.insert(key.clone()) {
                return Err(format!(
                    "duplicate key '{key}' in registration request"
                ));
            }

            if owners.contains_key(key) {
                return Err(format!(
                    "placeholder '{key}' is already registered"
                ));
            }
        }

        let count = keys.len();

        for key in keys {
            owners.insert(key, owner.to_owned());
        }

        Ok(count)
    }

    /// Returns the provider that owns a key.
    pub fn owner(&self, key: &NamespacedKey) -> Option<String> {
        self.owners
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .get(key)
            .cloned()
    }

    /// Removes selected keys, but only when the caller owns them.
    pub fn unregister_many(
        &self,
        owner: &str,
        keys: &[NamespacedKey]
    ) -> usize {
        let mut owners = self
            .owners
            .write()
            .unwrap_or_else(|error| error.into_inner());

        let mut removed = 0;

        for key in keys {
            let is_owner = owners
                .get(key)
                .is_some_and(|registered_owner| registered_owner == owner);

            if is_owner && owners.remove(key).is_some() {
                removed += 1;
            }
        }

        removed
    }

    /// Removes every key owned by a specific plugin.
    pub fn unregister_all(&self, owner: &str) -> usize {
        let mut owners = self
            .owners
            .write()
            .unwrap_or_else(|error| error.into_inner());
        
        let before = owners.len();

        owners.retain(|_, registered_owner| registered_owner != owner);

        before - owners.len()
    }
}