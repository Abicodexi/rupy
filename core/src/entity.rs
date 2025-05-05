use crate::CacheKey;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entity(pub usize);

impl Into<CacheKey> for Entity {
    fn into(self) -> CacheKey {
        CacheKey::new(self.0.to_string())
    }
}
