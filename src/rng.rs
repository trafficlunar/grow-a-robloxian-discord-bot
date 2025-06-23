// RobloxRng simulates math.random()
// https://github.com/luau-lang/luau/blob/640ebbc0a51bef0daa7c9b8c943d522dacb6a9a8/VM/src/lmathlib.cpp
// P.S. ChatGPT helped

const PCG32_INC: u64 = 105;

pub struct RobloxRng {
    state: u64,
}

impl RobloxRng {
    pub fn new(seed: u64) -> Self {
        let mut rng = RobloxRng { state: 0 };
        rng.step();
        rng.state = rng.state.wrapping_add(seed);
        rng.step();
        rng
    }

    fn step(&mut self) -> u32 {
        let oldstate = self.state;
        self.state = oldstate
            .wrapping_mul(6364136223846793005)
            .wrapping_add(PCG32_INC | 1);
        let xorshifted = (((oldstate >> 18) ^ oldstate) >> 27) as u32;
        let rot = (oldstate >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    // This behaves like math.random()
    pub fn next_f64(&mut self) -> f64 {
        let lo = self.step() as u64;
        let hi = self.step() as u64;

        let combined = lo | (hi << 32);
        (combined as f64) * (2f64).powi(-64)
    }

    /// This behaves like math.random(low, high)
    pub fn next_range(&mut self, low: u32, high: u32) -> u32 {
        let range = high - low;
        let test = (range as u64 + 1) * self.step() as u64;
        low + (test >> 32) as u32
    }
}
