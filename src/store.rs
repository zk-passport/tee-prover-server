use std::collections::HashMap;

pub trait Store {
    fn insert_new_agreement(
        &mut self,
        uuid: uuid::Uuid,
        shared_secret: Vec<u8>,
    ) -> Result<(), ring::error::Unspecified>;

    fn get_shared_secret(&self, uuid: &String) -> Option<Vec<u8>>;
}

pub struct HashMapStore {
    ecdh_store: HashMap<String, Vec<u8>>,
}

impl HashMapStore {
    pub fn new() -> Self {
        HashMapStore {
            ecdh_store: HashMap::new(),
        }
    }
}

impl Store for HashMapStore {
    fn insert_new_agreement(
        &mut self,
        uuid: uuid::Uuid,
        shared_secret: Vec<u8>,
    ) -> Result<(), ring::error::Unspecified> {
        self.ecdh_store.insert(uuid.to_string(), shared_secret);
        Ok(())
    }

    fn get_shared_secret(&self, uuid: &String) -> Option<Vec<u8>> {
        self.ecdh_store.get(uuid).cloned()
    }
}
