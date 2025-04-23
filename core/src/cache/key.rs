use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheKey {
    pub id: String,
}

impl CacheKey {
    pub fn new(id: impl Into<String>) -> Self {
        CacheKey { id: id.into() }
    }
}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

pub trait CacheKeyProvider {
    fn cache_key(&self) -> CacheKey;
}

impl Into<CacheKey> for &str {
    fn into(self) -> CacheKey {
        CacheKey::new(String::from(self))
    }
}
