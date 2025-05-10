use std::hash::{Hash, Hasher};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct CacheKey {
    id: u64,
}
impl CacheKey {
    pub fn hash<T: Hash>(value: T) -> u64 {
        let mut hasher = std::hash::DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
    pub fn id(&self) -> u64 {
        self.id
    }
}
impl CacheKey {
    pub fn new(id: impl Into<u64>) -> Self {
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
        CacheKey::new(CacheKey::hash(value))
    }
}
impl From<&str> for CacheKey {
    fn from(value: &str) -> Self {
        CacheKey::new(CacheKey::hash(value))
    }
}
impl From<crate::Entity> for CacheKey {
    fn from(value: crate::Entity) -> Self {
        Self { id: value.0 as u64 }
    }
}
impl Into<crate::Renderable> for CacheKey {
    fn into(self) -> crate::Renderable {
        crate::Renderable {
            model_key: self,
            visible: true,
        }
    }
}
impl Into<crate::Renderable> for &CacheKey {
    fn into(self) -> crate::Renderable {
        crate::Renderable {
            model_key: self.clone(),
            visible: true,
        }
    }
}
