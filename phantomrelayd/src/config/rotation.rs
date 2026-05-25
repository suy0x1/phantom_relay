pub struct RotationConfig {
    pub rotate_sec: u64,
}

impl RotationConfig {
    pub fn default() -> Self {
        Self { rotate_sec: 60 }
    }
}
