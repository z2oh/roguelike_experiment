use std::collections::HashMap;
use crate::util::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaterialId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tick(u64);

impl std::ops::Add<u64> for Tick {
    type Output = Self;

    fn add(self, offset: u64) -> Self::Output {
        Self(self.0 + offset)
    }
}

impl std::ops::Sub<u64> for Tick {
    type Output = Self;

    fn sub(self, offset: u64) -> Self::Output {
        Self(self.0 - offset)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockFill {
    /// This block is filled solid.
    Solid(MaterialId),
    /// This block is filled in only at the bottom.
    Floor(MaterialId),
    /// This block is filled in only at the top.
    Ceiling(MaterialId),
    /// This block is filled at the top and the bottom.
    FloorCeiling(MaterialId, MaterialId),
    /// This block is completely empty.
    Empty,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub fill: BlockFill,
}

#[derive(Clone, Debug)]
pub struct Region {
    /// This vector is guaranteed to be REGION_DIM^2 in size.
    pub blocks: Vec<Block>,
}

#[derive(Clone, Debug)]
pub struct CachedRegion {
    pub region: Region,
    /// The last tick on which this region was updated in world memory. This is useful for caching
    /// world state on render.
    pub last_update_tick: Tick,
}

#[derive(Clone, Debug)]
pub struct World {
    pub id: WorldId,
    /// Regions currently loaded into memory.
    regions: HashMap<(i32, i32, i32), CachedRegion>,

    /// The current tick of the simulated world. There are 1000 ticks in a given turn. If a player
    /// makes 10,000 turns per second (a massive overestimate), then a world may safely be simulated
    /// for over 50,000 consecutive years before overflow becomes a concern.
    pub current_tick: Tick,
}

impl World {
    pub fn new() -> Self {
        // TODO: Remove this camera; this is only here so we can generate a debug map that fills
        // the screen.
        let camera = crate::gfx::camera::Camera {
            world_offset: (0, 0, 0),
            region_offset: (0, 0),
            // TODO: Assuming glyph size of (10, 20) and window size of (1280, 720).
            tiles_dims: (128, 36),
        };

        Self {
            id: WorldId(0),
            regions: DEBUG_gen_regions(&camera),
            current_tick: Tick(1),
        }
    }

    pub fn get_cached_region(&self, offset: (i32, i32, i32)) -> Option<&CachedRegion> {
        self.regions.get(&offset)
    }
}

//
// TEMP/DEBUG MAP GENERATION CODE
//

use rand::distributions::{Distribution, Uniform};

#[allow(non_snake_case)]
fn DEBUG_gen_regions(camera: &crate::gfx::camera::Camera) -> HashMap<(i32, i32, i32), CachedRegion> {
    let (num_tiles_x, num_tiles_y) = camera.tiles_dims;

    let mut map_builder = MapBuilder {
        width: num_tiles_x,
        height: num_tiles_y,
        rooms: Vec::new(),
    };

    for _ in 0..20 {
        map_builder.add_random_room();
    }

    map_builder.build()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rect {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Rect {
    fn overlaps(&self, other: Rect) -> bool {
        self.x1 < other.x2 && self.x2 > other.x1 && self.y1 > other.y2 && self.y2 < other.y1
    }
}

struct MapBuilder {
    width: u32,
    height: u32,
    rooms: Vec<Rect>,
}

impl MapBuilder {
    fn add_room(&mut self, x: i32, y: i32, width: u32, height: u32) -> bool {
        let r = Rect {
            x1: x,
            y1: y,
            x2: x + width as i32,
            y2: y + height as i32,
        };

        if r.x2 > self.width as i32 || r.y2 > self.height as i32 { return false };

        if let Some(_) = self.rooms.iter().find(|&&room| room.overlaps(r)) {
            false
        } else {
            self.rooms.push(r);
            true
        }
    }

    fn add_random_room(&mut self) -> bool {
        let mut rng = rand::thread_rng();

        let x = Uniform::from(0..self.width);
        let y = Uniform::from(0..self.height);
        let width = Uniform::from(4..12);
        let height = Uniform::from(3..6);

        let mut tries: u32 = 0;

        loop {
            let rand_x = x.sample(&mut rng);
            let rand_y = y.sample(&mut rng);

            let rand_width = width.sample(&mut rng);
            let rand_height = height.sample(&mut rng);

            if self.add_room(rand_x as i32, rand_y as i32, rand_width, rand_height) {
                return true
            } else if tries >= 200 {
                return false
            } else {
                tries += 1;
                continue
            }
        }
    }

    fn build(self) -> HashMap<(i32, i32, i32), CachedRegion> {
        let mut map = HashMap::new();

        let mut flat_map = vec!['#'; (self.width * self.height) as usize];

        for room in self.rooms {
            for y in room.y1..room.y2 {
                for x in room.x1..room.x2 {
                    flat_map[(y * self.width as i32 + x) as usize] = '.';
                }
            }
        }

        let regions_wide = (self.width / REGION_DIM as u32) + 1;
        let regions_tall = (self.height / REGION_DIM as u32) + 1;

        for ry in 0..regions_tall {
            for rx in 0..regions_wide {
                let mut blocks = vec![ Block { fill: BlockFill::Solid(MaterialId(0)) }; REGION_LEN];
                for y in 0..REGION_DIM {
                    for x in 0..REGION_DIM {
                        let global_y = ry * (REGION_DIM as u32) + y as u32;
                        let global_x = rx * (REGION_DIM as u32) + x as u32;

                        let idx: usize = (global_y * self.width + global_x) as usize;

                        if idx >= flat_map.len() {
                            continue
                        } else {
                            if flat_map[idx] == '.' {
                                blocks[(y * REGION_DIM + x) as usize] = Block {
                                    fill: BlockFill::Floor(MaterialId(0)),
                                };
                            }
                        }
                    }
                }

                map.insert((rx as i32, ry as i32, 0), CachedRegion {
                    region: Region {
                        blocks,
                    },
                    last_update_tick: Tick(1),
                });
            }
        }

        map
    }
}
