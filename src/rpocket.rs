use crate::{store, store::auth_storage};

pub struct PocketBase<'a> {
    auth_state: auth_storage::AuthStorage<'a>,
}

impl<'a> PocketBase<'a> {
    /// Create a new PocketBase.
    /// storage: the storage to use.
    pub fn new(storage: &'a dyn store::Storage) -> Self {
        return crate::rpocket::PocketBase::new_with_auth_state(auth_storage::AuthStorage::new(storage));
    }

    /// Create a new PocketBase with an existing AuthStorage.
    pub fn new_with_auth_state(auth_state: auth_storage::AuthStorage<'a>) -> Self {
        return PocketBase { auth_state };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pocket_base() {
        let store = crate::store::MemoryStorage::new();
        let pocket_base = PocketBase::new(&store);
        assert!(pocket_base.auth_state.token().unwrap().is_none());
    }
}
