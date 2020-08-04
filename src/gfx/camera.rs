use crate::util::prelude::*;

/// The camera is just a tool to convert world coordinates to screen coordinates.
pub struct Camera {
    pub world_offset: (i32, i32, i32),
    pub region_offset: (u8, u8),
    pub tiles_dims: (u32, u32),
}

impl Camera {
    pub fn get_screen_coords(&self, (x, y, _): (i32, i32, i32)) -> (i32, i32) {
        let x_offset = ((x - self.world_offset.0) * REGION_DIM as i32) + self.region_offset.0 as i32;
        let y_offset = ((y - self.world_offset.1) * REGION_DIM as i32) + self.region_offset.1 as i32;

        (x_offset, y_offset)
    }
}
