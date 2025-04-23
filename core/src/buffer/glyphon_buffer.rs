use crate::{CacheKey, CacheKeyProvider, CacheStorage};

/// Wrapper around Glyphon buffers
pub struct GlyphonBuffer {
    pub buffer: glyphon::Buffer,
}

impl GlyphonBuffer {
    /// Create a new Glyphon buffer (atlas) with the given `FontSystem`
    pub fn new(font_system: &mut glyphon::FontSystem, metrics: Option<glyphon::Metrics>) -> Self {
        let buffer =
            glyphon::Buffer::new(font_system, metrics.unwrap_or(glyphon::Metrics::default()));
        GlyphonBuffer { buffer }
    }
    /// Create a Glyphon buffer from explicit metrics and pre-populated lines
    pub fn from_data(
        font_system: &mut glyphon::FontSystem,
        metrics: glyphon::Metrics,
        lines: &Vec<glyphon::BufferLine>,
    ) -> Self {
        let mut buffer = GlyphonBuffer::new(font_system, Some(metrics));
        buffer.push_buffer_lines(lines);
        buffer
    }

    /// Append lines into the buffer
    pub fn push_buffer_lines(&mut self, lines: &Vec<glyphon::BufferLine>) {
        for line in lines.iter() {
            self.buffer.lines.push(line.clone());
        }
    }

    /// Clear all lines from the buffer
    pub fn flush_buffer_lines(&mut self) {
        self.buffer.lines.clear();
        self.buffer.lines.shrink_to_fit();
    }
}

pub type GlyphonBufferCacheType = std::collections::HashMap<CacheKey, GlyphonBuffer>;
pub struct GlyphonBufferCache {
    inner: GlyphonBufferCacheType,
}

impl GlyphonBufferCache {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn get_or_create_buffer<K, F>(&mut self, key_source: &K, create_fn: F) -> &mut GlyphonBuffer
    where
        K: CacheKeyProvider,
        F: FnOnce() -> GlyphonBuffer,
    {
        let key = key_source.cache_key();
        self.inner.get_or_create(key, create_fn)
    }
}
