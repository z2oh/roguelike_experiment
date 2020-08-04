use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, GlyphCruncher, Section, Text};

use crate::gfx::gpu_context::GpuContext;

pub struct GlyphContext {
    // TODO: Load fonts as part of a resource manager. Arcs can be handed out from there.
    pub font: wgpu_glyph::ab_glyph::FontArc,
    pub glyph_brush: wgpu_glyph::GlyphBrush<()>,
}

impl GlyphContext {
    pub fn new(ttf_bytes: Vec<u8>, gpu_context: &GpuContext) -> Option<Self> {
        // TODO: Load fonts as part of a resource manager. Arcs can be handed out from there.
        let font = ab_glyph::FontArc::try_from_vec(ttf_bytes).ok()?;

        let glyph_brush = GlyphBrushBuilder::using_font(font.clone())
            .build(gpu_context.device(), gpu_context.render_format());

        Some(Self {
            font,
            glyph_brush,
        })
    }
}

pub struct MonospaceGlyphContext {
    // TODO: Should this wrap a glyph context as it does now, or can we treat these polymorphically
    // somehow?
    pub glyph_context: GlyphContext,

    // TODO: Because we store the scale here, we need to interface all `Text::new` ops from this
    // context, otherwise the scale used can be out of sync with the scale stored in the context.
    pub glyph_scale: (f32, f32),
    pub glyph_width: f32,
    pub glyph_height: f32,
}

impl MonospaceGlyphContext {
    pub fn new(
        ttf_bytes: Vec<u8>,
        glyph_scale: (f32, f32),
        gpu_context: &GpuContext,
    ) -> Option<Self> {
        let mut glyph_context = GlyphContext::new(ttf_bytes, gpu_context)?;

        let (glyph_width, glyph_height) = Self::calcuate_glyph_size(&mut glyph_context, glyph_scale)?;

        Some(Self {
            glyph_context,
            glyph_scale,
            glyph_width,
            glyph_height,
        })
    }

    fn calcuate_glyph_size(
        glyph_context: &mut GlyphContext,
        glyph_scale: (f32, f32),
    ) -> Option<(f32, f32)> {
        let px_scale = ab_glyph::PxScale {
            x: glyph_scale.0,
            y: glyph_scale.1,
        };

        let size_section = Section::default().add_text(Text::new("x").with_scale(px_scale));

        glyph_context.glyph_brush.glyph_bounds(size_section).map(|d| (d.max.x, d.max.y))
    }

    pub fn with_scale(&mut self, glyph_scale: (f32, f32)) {
        self.glyph_scale = glyph_scale;
        self.update_glyph_size();
    }

    pub fn get_px_scale(&self) -> ab_glyph::PxScale {
        ab_glyph::PxScale {
            x: self.glyph_scale.0,
            y: self.glyph_scale.1,
        }
    }

    fn update_glyph_size(&mut self) {
        let (width, height) =
            // TODO: Can this fail? Panicking is fine for now...
            Self::calcuate_glyph_size(&mut self.glyph_context, self.glyph_scale).unwrap();

        self.glyph_width = width;
        self.glyph_height = height;
    }
}
