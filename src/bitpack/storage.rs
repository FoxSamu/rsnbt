use std::fmt::Debug;

#[derive(Clone)]
pub struct BitStorage {
    raw: Vec<u64>,
    len: usize,
    wdt: usize,
    rpw: usize
}

impl BitStorage {
    pub const fn new(bitwidth: u8) -> Self {
        if bitwidth > 64 {
            panic!("Invalid bitwidth: must be at most 64");
        }

        Self {
            raw: Vec::new(),
            len: 0,
            wdt: bitwidth as usize,
            rpw: if bitwidth == 0 { usize::MAX } else { 64 / bitwidth as usize }
        }
    }

    pub fn from_raw<I>(bitwidth: u8, len: usize, raw: I) -> Self where I : IntoIterator<Item = u64> {
        if len >= (usize::MAX >> 6) {
            panic!("max size exceeded");
        }

        if bitwidth == 0 {
            return Self {
                raw: Vec::new(),
                len,
                wdt: 0,
                rpw: usize::MAX
            };
        }

        let raw = Vec::from_iter(raw);
        let wdt = bitwidth as usize;

        let rpw = if wdt == 0 { usize::MAX } else { 64 / wdt };

        let mut s = Self {
            raw,
            len,
            wdt,
            rpw
        };

        if s.len > s.raw.len() * rpw {
            s.grow_raw();
        } else {
            s.shrink_raw();
        }

        s
    }

    pub fn raw(&self) -> &[u64] {
        self.raw.as_slice()
    }

    pub fn raw_mut(&mut self) -> &mut [u64] {
        self.raw.as_mut_slice()
    }

    pub fn into_raw(self) -> Vec<u64> {
        self.raw
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn bitwidth(&self) -> u8 {
        self.wdt as u8
    }

    pub fn get(&self, index: usize) -> u64 {
        if index >= self.len {
            panic!("index out of bounds: {} - length is: {}", index, self.len)
        }

        if self.wdt == 0 {
            return 0;
        }

        let wdt = self.wdt;
        let rpw = self.rpw;
        let msk = (1 << wdt) - 1;

        let idx = index / rpw;
        let off = (index % rpw) * wdt;

        let raw = self.raw[idx];

        (raw >> off) & msk
    }

    pub fn set(&mut self, index: usize, bits: u64) {
        if index >= self.len {
            panic!("index out of bounds: {} - length is: {}", index, self.len)
        }

        if self.wdt == 0 {
            return;
        }

        let wdt = self.wdt;
        let rpw = self.rpw;
        let msk = (1 << wdt) - 1;

        let dat = bits & msk;
        
        let idx = index / rpw;
        let off = (index % rpw) * wdt;
    
        let mut raw = self.raw[idx];

        raw &= !(msk << off);
        raw |= dat << off;

        self.raw[idx] = raw;
    }

    pub fn push(&mut self, bits: u64) {
        if self.len == (usize::MAX >> 6) {
            panic!("max size exceeded");
        }

        let index = self.len;

        self.len += 1;
        self.grow_raw();

        self.set(index, bits);
    }

    pub fn pop(&mut self) -> Option<u64> {
        if self.len == 0 {
            return None;
        }

        let index = self.len - 1;
        let bits = self.get(index);

        self.len -= 1;
        self.shrink_raw();

        Some(bits)
    }

    pub fn remove(&mut self, index: usize) -> u64 {
        if index >= self.len {
            panic!("index out of bounds: {} - length is: {}", index, self.len)
        }

        let bits = self.get(index);

        self.copy_from_self(index + 1, index, self.len - (index + 1));
        bits
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.raw.clear();
    }

    pub fn extend_from(&mut self, src: &BitStorage, src_off: usize, len: usize) {
        let src_last = src_off + len;

        if src_last > src.len {
            panic!("source range is out of bounds: {}..{} - length is {}", src_off, src_last, src.len);
        }

        if self.len + len > (usize::MAX >> 6) {
            panic!("max size exceeded");
        }

        let dst = self.len;

        self.len += len;
        self.grow_raw();

        self.copy_from(src, src_off, dst, len);
    }

    pub fn extend_from_self(&mut self, src_off: usize, len: usize) {
        let src_last = src_off + len;

        if src_last > self.len {
            panic!("source range is out of bounds: {}..{} - length is {}", src_off, src_last, self.len);
        }

        if self.len + len > (usize::MAX >> 6) {
            panic!("max size exceeded");
        }

        let dst = self.len;

        self.len += len;
        self.grow_raw();

        self.copy_from_self(src_off, dst, len);
    }

    pub fn copy_from(&mut self, src: &BitStorage, src_off: usize, dst_off: usize, len: usize) {
        let src_last = src_off + len;
        let dst_last = dst_off + len;

        if src_last > src.len {
            panic!("source range is out of bounds: {}..{} - length is {}", src_off, src_last, src.len);
        }

        if dst_last > self.len {
            panic!("destination range is out of bounds: {}..{} - length is {}", dst_off, dst_last, self.len);
        }

        if len == 0 {
            return;
        }

        for i in 0..len {
            let bits = src.get(src_off + i);
            self.set(dst_off + i, bits);
        }
    }

    pub fn copy_from_self(&mut self, src_off: usize, dst_off: usize, len: usize) {
        let src_last = src_off + len;
        let dst_last = dst_off + len;

        if src_last > self.len {
            panic!("source range is out of bounds: {}..{} - length is {}", src_off, src_last, self.len);
        }

        if dst_last > self.len {
            panic!("destination range is out of bounds: {}..{} - length is {}", dst_off, dst_last, self.len);
        }

        if len == 0 {
            return;
        }

        if dst_off < src_off {
            // Backwards copy, copy left to right
            for i in 0..len {
                let bits = self.get(src_off + i);
                self.set(dst_off + i, bits);
            }
        } else {
            // Forwards copy, copy right to left
            for j in 1..=len {
                let i = len - j;

                let bits = self.get(src_off + i);
                self.set(dst_off + i, bits);
            }
        }
    }

    fn shrink_raw(&mut self) {
        if self.wdt == 0 {
            self.raw.clear();
            return;
        }

        while self.raw.len() * self.rpw >= self.len + self.rpw {
            self.raw.pop();
        }
    }

    fn grow_raw(&mut self) {
        if self.wdt == 0 {
            return;
        }

        while self.raw.len() * self.rpw < self.len {
            self.raw.push(0);
        }
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter {
            storage: self,
            index: 0
        }
    }

    pub fn rescale(&mut self, bitwidth: u8) {
        if bitwidth as usize == self.wdt {
            return;
        }
        
        let mut copy = BitStorage::new(bitwidth);
        copy.extend_from(self, 0, self.len);
        *self = copy;
    }

    pub fn max_bitwidth(&self) -> u8 {
        if self.wdt == 0 {
            return 0;
        }

        let mut wdt = 0usize;
        let mut msk = 0u64;

        'f:
        for i in 0..self.len {
            let bits = self.get(i);

            while (bits & !msk) != 0 {
                wdt += 1;
                if wdt == self.wdt {
                    break 'f;
                }

                msk = (1 << wdt) - 1;
            }
        }

        wdt as u8
    }

    pub fn compress(&mut self) -> u8 {
        let wdt = self.max_bitwidth();
        self.rescale(wdt);
        wdt
    }

    pub fn resize(&mut self, len: usize) {
        if len == self.len {
            return;
        }

        if len < self.len {
            self.len = len;
            self.shrink_raw();
            return;
        }

        if len > (usize::MAX >> 6) {
            panic!("max size exceeded");
        }

        self.len = len;
        self.grow_raw();
        return;
    }
}

impl PartialEq for BitStorage {
    fn eq(&self, other: &Self) -> bool {
        if !(self.len == other.len && self.wdt == other.wdt) {
            return false;
        }

        for i in 0..self.len {
            if self.get(i) != other.get(i) {
                return false;
            }
        }

        true
    }
}

impl Eq for BitStorage {
}

impl IntoIterator for BitStorage {
    type Item = u64;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        IntoIter {
            storage: self,
            index: 0
        }
    }
}


pub struct Iter<'a> {
    storage: &'a BitStorage,
    index: usize
}

impl Iterator for Iter<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.storage.len {
            None
        } else {
            let i = self.index;
            self.index += 1;
            Some(self.storage.get(i))
        }
    }
}


impl Debug for BitStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BitStorage [\n    ")?;

        let raw = &self.raw;
        for i in 0..raw.len() {
            let v = raw[i];
            write!(f, "{:016X}", v)?;

            if i == raw.len() - 1 {
                write!(f, "\n")?;
            } else if i & 3 == 3 {
                write!(f, "\n    ")?;
            } else {
                write!(f, " ")?;
            }
        }

        write!(f, "]")?;

        Ok(())
    }
}


pub struct IntoIter {
    storage: BitStorage,
    index: usize
}

impl Iterator for IntoIter {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.storage.len {
            None
        } else {
            let i = self.index;
            self.index += 1;
            Some(self.storage.get(i))
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let mut storage = BitStorage::new(3);

        storage.push(2);
        storage.push(4);
        storage.push(6);
        storage.push(8);

        assert_eq!(2, storage.get(0));
        assert_eq!(4, storage.get(1));
        assert_eq!(6, storage.get(2));
        assert_eq!(0, storage.get(3));
    }

    #[test]
    fn wide() {
        let mut storage = BitStorage::new(48);

        storage.push(0xC0FFEE);
        storage.push(0xDEDFCE);
        storage.push(0x1E42E1);
        storage.push(0x987654);

        assert_eq!(0xC0FFEE, storage.get(0));
        assert_eq!(0xDEDFCE, storage.get(1));
        assert_eq!(0x1E42E1, storage.get(2));
        assert_eq!(0x987654, storage.get(3));
    }
}