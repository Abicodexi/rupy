use crate::{CacheKey, CacheStorage, HashCache};
/// Wrapper around Glyphon buffers
pub struct GlyphonBuffer {
    pub buffer: glyphon::Buffer,
}

impl GlyphonBuffer {
    /// Create a new Glyphon buffer (atlas) with the given `FontSystem`
    pub fn new(font_system: &mut glyphon::FontSystem, metrics: Option<glyphon::Metrics>) -> Self {
        let buffer = glyphon::Buffer::new(
            font_system,
            metrics.unwrap_or(glyphon::Metrics::new(20.0, 20.0)),
        );
        GlyphonBuffer { buffer }
    }
    /// Create a Glyphon buffer from explicit metrics and pre-populated lines
    pub fn from_data(
        font_system: &mut glyphon::FontSystem,
        metrics: glyphon::Metrics,
        lines: &Vec<glyphon::BufferLine>,
    ) -> Self {
        let mut buffer = GlyphonBuffer::new(font_system, Some(metrics));
        buffer.push_lines(lines);
        buffer
    }
    /// Append lines into the buffer
    pub fn push_lines(&mut self, lines: &Vec<glyphon::BufferLine>) {
        for line in lines.iter() {
            self.buffer.lines.push(line.clone());
        }
    }
    /// Append lines into the buffer
    pub fn push_line(&mut self, line: glyphon::BufferLine) {
        self.buffer.lines.push(line);
    }
    pub fn set_lines(&mut self, lines: Vec<glyphon::BufferLine>) {
        self.buffer.lines = lines;
    }
    /// Clear all lines from the buffer
    pub fn clear_lines(&mut self) {
        self.buffer.lines.clear();
    }
    /// Clear all lines from the buffer
    pub fn shape(&mut self, font_system: &mut glyphon::FontSystem) {
        self.buffer.shape_until_scroll(font_system, false);
    }
}

pub type GlyphonBufferCacheType = HashCache<GlyphonBuffer>;
pub struct GlyphonBufferManager {
    inner: GlyphonBufferCacheType,
}

impl GlyphonBufferManager {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl CacheStorage<GlyphonBuffer> for GlyphonBufferManager {
    fn get(&self, key: &CacheKey) -> Option<&GlyphonBuffer> {
        self.inner.get(&key)
    }
    fn contains(&self, key: &CacheKey) -> bool {
        self.inner.contains_key(key)
    }
    fn get_mut(&mut self, key: &CacheKey) -> Option<&mut GlyphonBuffer> {
        self.inner.get_mut(key)
    }
    fn get_or_create<F>(&mut self, key: CacheKey, create_fn: F) -> &mut GlyphonBuffer
    where
        F: FnOnce() -> GlyphonBuffer,
    {
        self.inner.entry(key).or_insert_with(create_fn)
    }
    fn insert(&mut self, key: CacheKey, resource: GlyphonBuffer) {
        self.inner.insert(key, resource);
    }
    fn remove(&mut self, key: &CacheKey) {
        self.inner.remove(key);
    }
}
