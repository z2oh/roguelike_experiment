//! The abstraction layer before actually rendering glyphs. The world is "captured" into a scene
//! here and then can be rendered independent of world state. Until a turn is made, the world state
//! does not need to be recaptured. All non-stateful graphical elements (animations, etc.) are
//! captured by the types in this module.

pub struct GfxGlyph {
    pub glyph: &'static str,
    pub render_offset: [f32; 2],
}

impl GfxGlyph {
    pub fn new(glyph: &'static str) -> GfxGlyph {
        GfxGlyph {
            glyph,
            render_offset: [0.0, 0.0],
        }
    }
}

pub struct GfxTile {
    pub glyph: GfxGlyph,
    pub fg: [f32; 4],
    pub bg: [f32; 4],
}

pub struct GfxRegion {
    pub tiles: Vec<GfxTile>,
}

impl GfxRegion {
    /// Useful when using non-blocking (i.e. cache-only) world state retrieval functions.
    pub fn empty() -> Self {
        Self {
            tiles: Vec::new(),
        }
    }
}
