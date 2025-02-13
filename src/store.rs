use std::collections::HashMap;

pub trait Store {
    fn insert_new_agreement(
        &mut self,
        uuid: uuid::Uuid,
        shared_secret: Vec<u8>,
    ) -> Result<(), String>;

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
    ) -> Result<(), String> {
        if self.ecdh_store.contains_key(&uuid.to_string()) {
            return Err("Duplicate uuid".to_string());
        } else {
            self.ecdh_store.insert(uuid.to_string(), shared_secret);
        }

        return Ok(());
    }

    fn get_shared_secret(&self, uuid: &String) -> Option<Vec<u8>> {
        self.ecdh_store.get(uuid).cloned()
    }
}
