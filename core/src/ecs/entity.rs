#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Entity(pub usize);

impl Into<crate::CacheKey> for Entity {
    fn into(self) -> crate::CacheKey {
        crate::CacheKey::from(self.0.to_string())
    }
}
