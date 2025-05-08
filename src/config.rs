pub struct StateConfigs {
    pub base_zoom: f32,
    pub speed: f32,
    pub focal: f32,
    pub fov: f32,
    pub sensitivity: f32,
}

impl StateConfigs {
    pub fn default() -> Self {
        StateConfigs {
            base_zoom: 1.5,
            speed: 0.04,
            focal: 1.0,
            fov: 1.5708,
            sensitivity: 0.1,
        }
    }
}
