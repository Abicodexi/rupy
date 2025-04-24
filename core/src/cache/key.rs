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

impl From<String> for CacheKey {
    fn from(value: String) -> Self {
        CacheKey::new(value)
    }
}
impl From<&str> for CacheKey {
    fn from(value: &str) -> Self {
        CacheKey::new(value)
    }
}
