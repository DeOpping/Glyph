use std::{
    error::Error,
    fmt,
    str::FromStr
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamespacedKey {
    namespace: String,
    key: String
}

impl NamespacedKey {
    /// Creates a validated namespaced key.
    pub fn new(
        namespace: impl Into<String>,
        key: impl Into<String>
    ) -> Result<Self, NamespacedKeyError> {
        let namespace = namespace.into();
        let key = key.into();

        if namespace.is_empty() {
            return Err(NamespacedKeyError::EmptyNamespace);
        }

        if key.is_empty() {
            return Err(NamespacedKeyError::EmptyKey);
        }

        if !namespace.chars().all(is_valid_namespace_char) {
            return Err(NamespacedKeyError::InvalidNamespace);
        }

        if !key.chars().all(is_valid_key_char) {
            return Err(NamespacedKeyError::InvalidKey);
        }

        Ok(Self { namespace, key })
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn key(&self) -> &str {
        &self.key
    }

}

fn is_valid_namespace_char(character: char) -> bool {
    character.is_ascii_lowercase()
        || character.is_ascii_digit()
        || matches!(character, '_' | '-' | '.')
}

fn is_valid_key_char(character: char) -> bool {
    is_valid_namespace_char(character) || character == '/'
}

impl FromStr for NamespacedKey {
    type Err = NamespacedKeyError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (namespace, key) = value
            .split_once(':')
            .ok_or(NamespacedKeyError::MissingSeparator)?;

        if key.contains(':') {
            return Err(NamespacedKeyError::MultipleSeparators);
        }

        Self::new(namespace, key)
    }
}

impl fmt::Display for NamespacedKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.namespace, self.key)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NamespacedKeyError {
    MissingSeparator,
    MultipleSeparators,
    EmptyNamespace,
    EmptyKey,
    InvalidNamespace,
    InvalidKey
}

impl fmt::Display for NamespacedKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingSeparator => "expected namespace:key",
            Self::MultipleSeparators => {
                "a namespaced key cannot contain multiple colons"
            },
            Self::EmptyNamespace => "namespace cannot be empty",
            Self::EmptyKey => "key cannot be empty",
            Self::InvalidNamespace => "namespace contains invalid characters",
            Self::InvalidKey => "key contains invalid characters"
        };

        formatter.write_str(message)
    }
}

impl Error for NamespacedKeyError {}