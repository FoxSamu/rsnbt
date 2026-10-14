#[derive(Debug, Clone)]
pub struct BitArray {
    data: Vec<u64>
}

impl BitArray {
    pub const fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn get(&self, index: usize) -> bool {
        let ci = index >> 6;

        if ci >= self.data.len() {
            return false;
        }

        let cell = self.data[index >> 6];
        let off = index & 0b111111;
        let mask = 1 << off;

        (cell & mask) == mask
    }

    pub fn set(&mut self, index: usize) {
        let ci = index >> 6;

        if ci >= self.data.len() {
            let diff = (ci + 1) - self.data.len();
            self.data.append(&mut vec![0u64; diff]);
        }

        let off = index & 0b111111;
        let mask = 1 << off;

        self.data[ci] |= mask;
    }

    pub fn reset(&mut self, index: usize) {
        let ci = index >> 6;

        if ci >= self.data.len() {
            // There are no set bits here
            return;
        }

        let off = index & 0b111111;
        let mask = 1 << off;

        self.data[ci] &= !mask;

        // Remove trailing zeroes
        while let Some(0) = self.data.last() {
            self.data.pop();
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn set_range(&mut self, from: usize, to: usize) {
        for i in from..to {
            self.set(i);
        }
    }

    pub fn reset_range(&mut self, from: usize, to: usize) {
        for i in from..to {
            self.reset(i);
        }
    }

    pub fn find_gap(&mut self, start: usize, length: usize) -> usize {
        let mut i = start;
        let mut gap_start = start;

        loop {
            let cur_gap = i - gap_start;
            if cur_gap >= length {
                return gap_start;
            }

            if self.get(i) {
                gap_start = i + 1;
            }

            i += 1;
        }
    }

    pub fn is_gap(&mut self, from: usize, to: usize) -> bool {
        for i in from..to {
            if self.get(i) {
                return false;
            }
        }

        true
    }
}
