use crate::util::prelude::*;
use crate::state::prelude::*;
use crate::gfx;
use crate::gfx::glyph_gfx::*;

use std::collections::{HashMap, HashSet};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RenderModifier {
    GravityInverse,
}

struct CachedRegion {
    tick: Tick,
    region: GfxRegion
}

pub struct WorldRenderer {
    world_id: WorldId,
    render_modifiers: HashSet<RenderModifier>,
    render_cache: HashMap<(i32, i32, i32), CachedRegion>,
    camera: gfx::camera::Camera,
}

const OFF_SCREEN_RENDER_HEURISTIC: i32 = 2;

impl WorldRenderer {
    pub fn new(world_id: WorldId) -> Self {
        let camera = gfx::camera::Camera {
            world_offset: (0, 0, 0),
            region_offset: (0, 0),
            // TODO: Assuming glyph size of (10, 20) and window size of (1280, 720).
            tiles_dims: (128, 36),
        };

        Self {
            world_id,
            render_modifiers: HashSet::new(),
            render_cache: HashMap::new(),
            camera,
        }
    }

    pub fn render(
        &mut self,
        // TODO: Separate the world from the cache so that we don't need a mutable handle to the world.
        world: &mut World,
        glyph_context: &mut gfx::glyph_context::MonospaceGlyphContext,
    ) {
        assert!(self.world_id == world.id,
            "World renderer called with a different world than the one with which is was initialized.");

        // TODO: All this logic should be in the camera. probably.
        let (base_region_x, base_region_y, z) = self.camera.world_offset;
        let (render_size_x, render_size_y) = self.camera.tiles_dims;

        let regions_wide = ((render_size_x / REGION_DIM as u32) + 1) as i32;
        let regions_tall = ((render_size_y / REGION_DIM as u32) + 1) as i32;

        let base_y = base_region_y - OFF_SCREEN_RENDER_HEURISTIC;
        let final_y = base_region_y + regions_tall + OFF_SCREEN_RENDER_HEURISTIC;

        let base_x = base_region_x - OFF_SCREEN_RENDER_HEURISTIC;
        let final_x = base_region_x + regions_wide + OFF_SCREEN_RENDER_HEURISTIC;

        for y in base_y..final_y {
            for x in base_x..final_x {
                self.render_offset(world, glyph_context, (x, y, z));
            }
        }
    }

    fn render_offset(
        &mut self,
        world: &mut World,
        glyph_context: &mut gfx::glyph_context::MonospaceGlyphContext,
        offset: (i32, i32, i32),
    ) {
        let (tile_x, tile_y) = self.camera.get_screen_coords(offset);

        let region = &self.get_cached_region(world, offset).region;

        let mut section = wgpu_glyph::Section {
            screen_position: (
                tile_x as f32 * glyph_context.glyph_width,
                tile_y as f32 * glyph_context.glyph_height,
            ),
            ..wgpu_glyph::Section::default()
        };

        // Do not attempt to render a region if its `Tiles` vector is malformed.
        if region.tiles.len() != REGION_DIM as usize * REGION_DIM as usize {
            return;
        }

        for y in 0..REGION_DIM {
            for x in 0..REGION_DIM {
                let idx = (y as usize * REGION_DIM as usize) + x as usize;
                let tile = &region.tiles[idx];
                section = section.add_text(
                    wgpu_glyph::Text::new(tile.glyph.glyph)
                        .with_color(tile.fg)
                        .with_scale(glyph_context.get_px_scale())
                );
            }
            // TODO: Is this really the best way to do this?
            section = section.add_text(wgpu_glyph::Text::new("\n"));
        }

        glyph_context.glyph_context.glyph_brush.queue(section);
    }

    fn get_cached_region(
        &mut self,
        world: &World,
        offset: (i32, i32, i32),
    ) -> &CachedRegion {
        // TODO: This logic is haphazard.
        if let Some(world_region) = world.get_cached_region(offset) {
            let render_modifiers = &self.render_modifiers;
            self.render_cache.entry(offset)
                .and_modify(|cached_region| {
                    if world_region.last_update_tick > cached_region.tick {
                        *cached_region = CachedRegion {
                            region: gen_gfx_region(&world_region.region, render_modifiers),
                            tick: world.current_tick,
                        };
                    }
                })
                .or_insert_with(|| CachedRegion {
                    region: gen_gfx_region(&world_region.region, render_modifiers),
                    tick: world.current_tick,
                })
        } else {
            // The region wasn't loaded in memory, so there's no way we can render it. Just "render"
            // the empty region instead.
            self.render_cache.entry(offset)
                .and_modify(|cr| *cr = CachedRegion {
                    region: GfxRegion::empty(),
                    // Aggressively try to rerender this region.
                    tick: world.current_tick - 1,
                })
                .or_insert_with(|| CachedRegion {
                    region: GfxRegion::empty(),
                    // Aggressively try to rerender this region.
                    tick: world.current_tick - 1,
                })
        }
    }

    pub fn add_render_modifier(&mut self, render_modifier: RenderModifier) {
        self.render_cache.clear();
        self.render_modifiers.insert(render_modifier);
    }

    pub fn remove_render_modifier(&mut self, render_modifier: RenderModifier) {
        self.render_cache.clear();
        self.render_modifiers.remove(&render_modifier);
    }
}

fn gen_gfx_region(region: &Region, _render_modifiers: &HashSet<RenderModifier>) -> GfxRegion {
    let mut tiles = Vec::with_capacity(REGION_DIM as usize * REGION_DIM as usize);
    for b in &region.blocks {
        match b.fill {
            BlockFill::Solid(_m_id) => tiles.push(GfxTile {
                glyph: GfxGlyph::new("#"),
                fg: [1.0, 0.0, 0.0, 1.0],
                bg: [0.0, 1.0, 0.0, 1.0],
            }),
            BlockFill::Floor(_m_id) => tiles.push(GfxTile {
                glyph: GfxGlyph::new("."),
                fg: [0.0, 0.0, 1.0, 1.0],
                bg: [0.0, 0.0, 0.0, 1.0],
            }),
            _ => todo!(),
        }
    }

    GfxRegion { tiles }
}
