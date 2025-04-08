pub struct Rng {
    seed: u32,
}

impl Rng {
    pub fn new(seed: u32) -> Self {
        Rng { seed }
    }
    
    pub fn random_u32(&mut self) -> u32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        self.seed
    }

    pub fn random_range_u32(&mut self, min: u32, max: u32) -> u32 {
        min + (self.random_u32() % (max - min + 1))
    }
}