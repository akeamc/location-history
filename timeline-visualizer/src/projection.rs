use std::f32::consts::PI;

use location_history::protocol::LngLat;

pub struct CroppedWebMercator {
    width_px: u32,
    height_px: u32,
    lng_left: f32,
    lng_right: f32,
    lat_bottom: f32,
}

impl CroppedWebMercator {
    pub const fn new(
        width_px: u32,
        height_px: u32,
        lng_left: f32,
        lng_right: f32,
        lat_bottom: f32,
    ) -> Self {
        Self {
            width_px,
            height_px,
            lng_left,
            lng_right,
            lat_bottom,
        }
    }

    pub fn width(&self) -> u32 {
        self.width_px
    }

    pub fn height(&self) -> u32 {
        self.height_px
    }

    pub fn project(&self, LngLat(lng, lat): LngLat) -> (f32, f32) {
        let map_width_degrees = self.lng_right - self.lng_left;
        let world_map_width = ((self.width_px as f32 / map_width_degrees) * 360.0) / (2.0 * PI);

        fn ln_sin_something(x: f32) -> f32 {
            let sin_x = x.sin();
            ((1.0 + sin_x) / (1.0 - sin_x)).ln()
        }

        let map_offset_y = world_map_width / 2.0 * ln_sin_something(self.lat_bottom.to_radians());

        let x = (lng - self.lng_left) * (self.width_px as f32 / map_width_degrees);
        let y = self.height_px as f32
            - ((world_map_width / 2.0 * ln_sin_something(lat.to_radians())) - map_offset_y);

        (x, y)
    }

    /// Return `None` if the point is outside the map.
    pub fn project_int(&self, coords: LngLat) -> Option<(u32, u32)> {
        let (x, y) = self.project(coords);

        if x < 0.0 || x >= self.width_px as f32 {
            return None;
        }

        if y < 0.0 || y >= self.height_px as f32 {
            return None;
        }

        Some((x as _, y as _))
    }
}
