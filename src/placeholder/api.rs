use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::namespaced_key::NamespacedKey;

use super::parser;

type Resolver<C> = dyn Fn(&C) -> Option<String> + Send + Sync + 'static;

/// Registers and resolves namespaced placeholders.
///
/// 'C' represents the context supplied to each resolver.
pub struct PlaceholderApi<C = ()> {
    expansions: RwLock<HashMap<NamespacedKey, Arc<Resolver<C>>>>,
}

impl<C> PlaceholderApi<C> {
    /// Creates an empty placeholder API.
    pub fn new() -> Self {
        Self {
            expansions: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a resolver for one namespaced placeholder.
    ///
    /// Returns an error if the key is already registered.
    pub fn register<F>(&self, key: NamespacedKey, resolver: F) -> Result<(), RegistrationError>
    where
        F: Fn(&C) -> Option<String> + Send + Sync + 'static,
    {
        let mut expansions = self
            .expansions
            .write()
            .unwrap_or_else(|error| error.into_inner());

        if expansions.contains_key(&key) {
            return Err(RegistrationError::AlreadyRegistered(key));
        }

        expansions.insert(key, Arc::new(resolver));

        Ok(())
    }

    /// Unregisters a placeholder.
    ///
    /// Returns true if a registration was removed.
    pub fn unregister(&self, key: &NamespacedKey) -> bool {
        self.expansions
            .write()
            .unwrap_or_else(|error| error.into_inner())
            .remove(key)
            .is_some()
    }

    /// Checks whether a placeholder is registered.
    pub fn contains(&self, key: &NamespacedKey) -> bool {
        self.expansions
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .contains_key(key)
    }

    /// Returns the number of registered placeholders.
    pub fn len(&self) -> usize {
        self.expansions
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .len()
    }

    /// Returns whether there are no registered placeholders.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Parses a string using the supplied context.
    pub fn parse_with_context(&self, input: &str, context: &C) -> String {
        parser::parse(input, |key| {
            let resolver = {
                let expansions = self
                    .expansions
                    .read()
                    .unwrap_or_else(|error| error.into_inner());

                expansions.get(key).cloned()
            };

            resolver.and_then(|resolver| resolver(context))
        })
    }

    /// Resolves one specific placeholder.
    ///
    /// The resolver is cloned while holding the registry lock.
    /// User-provided code executes after the lock has been released.
    pub fn resolve(&self, key: &NamespacedKey, context: &C) -> Option<String> {
        let resolver = {
            let expansions = self
                .expansions
                .read()
                .unwrap_or_else(|error| error.into_inner());

            expansions.get(key).cloned()
        };

        resolver.and_then(|resolver| resolver(context))
    }
}

impl<C> Default for PlaceholderApi<C> {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience API for placeholders without a context.
impl PlaceholderApi<()> {
    pub fn parse(&self, input: &str) -> String {
        self.parse_with_context(input, &())
    }
}

/// Errors that occur while registering a placeholder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistrationError {
    AlreadyRegistered(NamespacedKey),
}

impl std::fmt::Display for RegistrationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyRegistered(key) => {
                write!(formatter, "placeholder '{key}' is already registered")
            }
        }
    }
}

impl std::error::Error for RegistrationError {}
