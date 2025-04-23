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

impl Into<CacheKey> for &str {
    fn into(self) -> CacheKey {
        CacheKey::new(String::from(self))
    }
}
impl Into<CacheKey> for String {
    fn into(self) -> CacheKey {
        CacheKey::new(self)
    }
}
