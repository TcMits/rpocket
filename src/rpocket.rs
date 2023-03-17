use crate::{store, store::auth_storage};

pub struct PocketBase<'a> {
    base_url: &'static str,
    lang: &'static str,
    auth_state: auth_storage::AuthStorage<'a>,
}

impl<'a> PocketBase<'a> {
    /// Create a new PocketBase client.
    /// base_url: the base URL for the PocketBase server.
    /// lang: the language to use.
    /// storage: the storage to use.
    pub fn new(
        base_url: &'static str,
        lang: &'static str,
        storage: &'a dyn store::Storage,
    ) -> Self {
        return crate::rpocket::PocketBase::new_with_auth_state(
            base_url,
            lang,
            auth_storage::AuthStorage::new(storage),
        );
    }

    /// Create a new PocketBase with an existing AuthStorage.
    /// base_url: the base URL for the PocketBase server.
    /// lang: the language to use.
    /// storage: the storage to use.
    pub fn new_with_auth_state(
        base_url: &'static str,
        lang: &'static str,
        auth_state: auth_storage::AuthStorage<'a>,
    ) -> Self {
        return PocketBase {
            base_url,
            lang,
            auth_state,
        };
    }

    /// Get the AuthStorage.
    pub fn auth_state(&self) -> &auth_storage::AuthStorage {
        return &self.auth_state;
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &'static str {
        return self.base_url;
    }

    /// Get the language.
    pub fn lang(&self) -> &'static str {
        return self.lang;
    }
}

#[cfg(test)]
mod test {
    // use super::*;
}
