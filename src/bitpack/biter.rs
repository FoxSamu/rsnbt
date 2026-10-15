/// A trait for iterators that are "biterable" (bit(wise) + iterable).
/// 
/// Biteration is a streaming alternatives to [`BitStorage`][super::BitStorage].
/// This is advantageous if you need to decode bit storages and then write the
/// data somewhere where its bit packing doesn't matter anymore. Where
/// [`BitStorage`][super::BitStorage] creates a buffer and has to read this buffer 
/// repeatedly, biteration only stores the minimum needed to pack or unpack bit
/// storage.
/// 
/// If your goal is to decode bit storage, modify it occasionally, and then reencode
/// it, then you are better off using [`BitStorage`][super::BitStorage].
/// 
/// This trait is implemented on any `Iterator<Item = u64>`, as well as every
/// `Iterator<Item = i64>` through [`Unsign`].
/// 
/// # Examples
/// ## Unpacking values
/// ```
/// use rsnbt::bitpack::Biter;
/// 
/// // Elements are packed from lowest to highest bit
/// let packed = vec![0xEDCBAu64];
/// 
/// let mut iter = packed.into_iter().unpack_exact(4, 5);
/// 
/// assert_eq!(Some(0xA), iter.next());
/// assert_eq!(Some(0xB), iter.next());
/// assert_eq!(Some(0xC), iter.next());
/// assert_eq!(Some(0xD), iter.next());
/// assert_eq!(Some(0xE), iter.next());
/// assert_eq!(None, iter.next());
/// ```
/// 
/// ## Packing values
/// ```
/// use rsnbt::bitpack::Biter;
/// 
/// // Elements are packed from lowest to highest bit
/// let packed = vec![0xAu64, 0xB, 0xC, 0xD, 0xE];
/// 
/// let mut iter = packed.into_iter().pack_exact(4, 5);
/// 
/// assert_eq!(Some(0xEDCBA), iter.next());
/// assert_eq!(None, iter.next());
/// ```
pub trait Biter<E> : Iterator<Item = E> {
    type Iter : Iterator<Item = u64>;

    /// Returns an iterator ([`Pack`]) that packs the values in this iterator with
    /// a given bitwidth. Once this iterator returns [`None`], the returned iterator
    /// will end and return [`None`] forever. It will stop consuming the underlying
    /// iterator after [`None`].
    /// 
    /// `bitwidth` must be at most 64. It is possible to set `bitwidth` to 0, in
    /// which case the returned iterator will consume all data entirely and return
    /// [`None`].
    /// 
    /// # Panics
    /// This method will panic when `bitwidth` is bigger than 64, since bitstrings
    /// of over 64 bits won't fit [`u64`].
    fn pack(self, bitwidth: u8) -> Pack<Self::Iter>;

    /// Returns an iterator ([`PackEndless`]) that packs the values in this iterator
    /// with a given bitwidth. This iterator assumes the underlying iterator will
    /// return more values after returning [`None`], so it will always return fully
    /// packed words, or [`None`] while waiting for more values from the underlying
    /// iterator. 
    /// 
    /// `bitwidth` must be at most 64. It is possible to set `bitwidth` to 0, in
    /// which case the returned iterator will consume all data entirely and always
    /// return [`None`].
    /// 
    /// # Panics
    /// This method will panic when `bitwidth` is bigger than 64, since bitstrings
    /// of over 64 bits won't fit [`u64`].
    fn pack_endless(self, bitwidth: u8) -> PackEndless<Self::Iter>;

    /// Returns an iterator ([`PackExact`]) that packs a given amount of values
    /// from this iterator with a given bitwidth. This iterator assumes the underlying
    /// iterator will return more values after returning [`None`], so it will return
    /// [`None`] whenever there isn't enough data to complete a word. The last word
    /// may not be fully filled if the length is reached.
    /// 
    /// `bitwidth` must be at most 64. It is possible to set `bitwidth` to 0, in
    /// which case the returned iterator will consume the underlying iterator until
    /// the requested amount of values is consumed, and always return [`None`].
    /// 
    /// # Panics
    /// This method will panic when `bitwidth` is bigger than 64, since bitstrings
    /// of over 64 bits won't fit [`u64`].
    fn pack_exact(self, bitwidth: u8, len: usize) -> PackExact<Self::Iter>;

    /// Returns an iterator ([`Unpack`]) that unpacks words from this iterator into
    /// values of a given btiwidth. For each time the underlying iterator returns
    /// [some][`Some`] value, the returned iterator will unpack all the values that
    /// fit the word. This means that any padding from the underlying storage will
    /// also be returned. This iterator will return [`Some`] after [`None`] if the
    /// underlying iterator does so.
    ///
    /// `bitwidth` must be at most 64. It is possible to set `bitwidth` to 0, in
    /// which case the returned iterator will return `None` until the underlying
    /// iterator returns some value, after which it will return `Some(0)` forever
    /// without consuming more values.
    /// 
    /// # Panics
    /// This method will panic when `bitwidth` is bigger than 64, since bitstrings
    /// of over 64 bits won't fit [`u64`].
    fn unpack(self, bitwidth: u8) -> Unpack<Self::Iter>;

    /// Returns an iterator ([`UnpackExact`]) that unpacks words from this iterator
    /// into values of a given btiwidth. For each time the underlying iterator
    /// returns [some][`Some`] value, the returned iterator will unpack all the values
    /// that fit the word until the given limit is reached. This assumes the underlying
    /// iterator will return more values after `None`. If the underlying iterator never
    /// produces enough words to reach the given limit, then the returned iterator will
    /// behave exactly like [`Biterable::unpack`].
    ///
    /// `bitwidth` must be at most 64. It is possible to set `bitwidth` to 0, in
    /// which case the returned iterator will return `None` until the underlying
    /// iterator returns some value, after which it will return `Some(0)` until the
    /// limit is reached, without consuming more values.
    /// 
    /// This iterator will return `None` forever once it returned the given limit
    /// of values.
    /// 
    /// # Panics
    /// This method will panic when `bitwidth` is bigger than 64, since bitstrings
    /// of over 64 bits won't fit [`u64`].
    fn unpack_exact(self, bitwidth: u8, len: usize) -> UnpackExact<Self::Iter>;
}

/// Implementation of [`Biter`] on iterators of [`u64`]s.
impl<I> Biter<u64> for I where I : Iterator<Item = u64> {
    type Iter = Self;

    fn unpack(self, bitwidth: u8) -> Unpack<Self> {
        if bitwidth > 64 {
            panic!("Invalid bitwidth: must be at most 64");
        }

        Unpack {
            iter: self,
            wdt: bitwidth as usize,
            rpw: if bitwidth == 0 { usize::MAX } else { 64 / bitwidth as usize },
            off: 0,
            cur: None
        }
    }

    fn unpack_exact(self, bitwidth: u8, len: usize) -> UnpackExact<Self> {
        if bitwidth > 64 {
            panic!("Invalid bitwidth: must be at most 64");
        }

        UnpackExact {
            iter: self,
            wdt: bitwidth as usize,
            rpw: if bitwidth == 0 { usize::MAX } else { 64 / bitwidth as usize },
            off: 0,
            cur: None,
            rem: len
        }
    }

    fn pack(self, bitwidth: u8) -> Pack<Self> {
        if bitwidth > 64 {
            panic!("Invalid bitwidth: must be at most 64");
        }

        Pack {
            iter: self,
            wdt: bitwidth as usize,
            rpw: if bitwidth == 0 { usize::MAX } else { 64 / bitwidth as usize },
            end: false
        }
    }

    fn pack_endless(self, bitwidth: u8) -> PackEndless<Self> {
        if bitwidth > 64 {
            panic!("Invalid bitwidth: must be at most 64");
        }

        PackEndless {
            iter: self,
            wdt: bitwidth as usize,
            rpw: if bitwidth == 0 { usize::MAX } else { 64 / bitwidth as usize },
            nxt: 0,
            off: 0
        }
    }

    fn pack_exact(self, bitwidth: u8, len: usize) -> PackExact<Self> {
        if bitwidth > 64 {
            panic!("Invalid bitwidth: must be at most 64");
        }

        PackExact {
            iter: self,
            wdt: bitwidth as usize,
            rpw: if bitwidth == 0 { usize::MAX } else { 64 / bitwidth as usize },
            nxt: 0,
            off: 0,
            rem: len
        }
    }
}


/// Implementation of [`Biter`] on iterators of [`i64`]s, using [`Unsign`] to
/// convert from [`i64`] to [`u64`].
impl<I> Biter<i64> for I where I : Iterator<Item = i64> {
    type Iter = Unsign<i64, Self>;

    fn unpack(self, bitwidth: u8) -> Unpack<Self::Iter> {
        if bitwidth > 64 {
            panic!("Invalid bitwidth: must be at most 64");
        }

        Unpack {
            iter: Unsign { iter: self },
            wdt: bitwidth as usize,
            rpw: if bitwidth == 0 { usize::MAX } else { 64 / bitwidth as usize },
            off: 0,
            cur: None
        }
    }

    fn unpack_exact(self, bitwidth: u8, len: usize) -> UnpackExact<Self::Iter> {
        if bitwidth > 64 {
            panic!("Invalid bitwidth: must be at most 64");
        }

        UnpackExact {
            iter: Unsign { iter: self },
            wdt: bitwidth as usize,
            rpw: if bitwidth == 0 { usize::MAX } else { 64 / bitwidth as usize },
            off: 0,
            cur: None,
            rem: len
        }
    }

    fn pack(self, bitwidth: u8) -> Pack<Self::Iter> {
        if bitwidth > 64 {
            panic!("Invalid bitwidth: must be at most 64");
        }

        Pack {
            iter: Unsign { iter: self },
            wdt: bitwidth as usize,
            rpw: if bitwidth == 0 { usize::MAX } else { 64 / bitwidth as usize },
            end: false
        }
    }

    fn pack_endless(self, bitwidth: u8) -> PackEndless<Self::Iter> {
        if bitwidth > 64 {
            panic!("Invalid bitwidth: must be at most 64");
        }

        PackEndless {
            iter: Unsign { iter: self },
            wdt: bitwidth as usize,
            rpw: if bitwidth == 0 { usize::MAX } else { 64 / bitwidth as usize },
            nxt: 0,
            off: 0
        }
    }

    fn pack_exact(self, bitwidth: u8, len: usize) -> PackExact<Self::Iter> {
        if bitwidth > 64 {
            panic!("Invalid bitwidth: must be at most 64");
        }

        PackExact {
            iter: Unsign { iter: self },
            wdt: bitwidth as usize,
            rpw: if bitwidth == 0 { usize::MAX } else { 64 / bitwidth as usize },
            nxt: 0,
            off: 0,
            rem: len
        }
    }
}



/// An iterator that converts signed integers to unsigned integers.
pub struct Unsign<E, I> where I : Iterator<Item = E> {
    iter: I
}

impl<I> Iterator for Unsign<i8, I> where I : Iterator<Item = i8> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|v| v as u8)
    }
}

impl<I> Iterator for Unsign<i16, I> where I : Iterator<Item = i16> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|v| v as u16)
    }
}

impl<I> Iterator for Unsign<i32, I> where I : Iterator<Item = i32> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|v| v as u32)
    }
}

impl<I> Iterator for Unsign<i64, I> where I : Iterator<Item = i64> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|v| v as u64)
    }
}

impl<I> Iterator for Unsign<isize, I> where I : Iterator<Item = isize> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|v| v as usize)
    }
}

impl<I> Iterator for Unsign<i128, I> where I : Iterator<Item = i128> {
    type Item = u128;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|v| v as u128)
    }
}


pub struct Pack<I> where I : Iterator<Item = u64> {
    iter: I,
    wdt: usize,
    rpw: usize,
    end: bool
}

impl<I> Iterator for Pack<I> where I : Iterator<Item = u64> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }

        if self.wdt == 0 {
            // bit width is zero so all values
            // will fit into 0 bits, leaving no
            // words in the resulting iterator
            while let Some(_) = self.iter.next() {
                // n/a
            }

            return None;
        }

        let mut nxt = 0u64;
        let rpw = self.rpw;
        let wdt = self.wdt;

        for i in 0..rpw {
            let Some(dat) = self.iter.next() else {
                self.end = true;

                if i == 0 {
                    // if it was supposed to be the first,
                    // then that means we don't have any
                    // trailing values in the current word
                    // and it is unnecessary
                    return None;
                } else {
                    // we have trailing data
                    return Some(nxt);
                }
            };

            nxt |= dat << (i * wdt);
        }

        Some(nxt)
    }
}


pub struct PackEndless<I> where I : Iterator<Item = u64> {
    iter: I,
    wdt: usize,
    rpw: usize,

    nxt: u64,
    off: usize
}

impl<I> Iterator for PackEndless<I> where I : Iterator<Item = u64> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.wdt == 0 {
            // our iterator is endless so we will
            // never be able to generate even one
            // word
            //
            // consume the iterator as expected
            while let Some(_) = self.iter.next() {
            }
            
            return None;
        }

        let off = self.off;

        // mask only the bits that are actually
        // relevant in the tail so the rest is cleared
        let mut nxt = self.nxt & ((1 << off) - 1);

        let rpw = self.rpw;
        let wdt = self.wdt;

        for i in off..rpw {
            let Some(dat) = self.iter.next() else {
                // store what we currently have
                self.nxt = nxt;
                self.off = i;
                return None;
            };

            nxt |= dat << (i * wdt);
        }

        Some(nxt)
    }
}



pub struct PackExact<I> where I : Iterator<Item = u64> {
    iter: I,
    wdt: usize,
    rpw: usize,

    nxt: u64,
    off: usize,
    rem: usize
}

impl<I> Iterator for PackExact<I> where I : Iterator<Item = u64> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rem == 0 {
            return None;
        }

        if self.wdt == 0 {
            // elements will all fit exactly 0 bits,
            // so let's consume exactly as many elements
            // as requested and then quit
            while let Some(_) = self.iter.next() {
                self.rem -= 1;
            }

            // 0 bits fits 0 words, so there is no
            // word to begin with
            return None;
        }

        let off = self.off;

        // mask only the bits that are actually
        // relevant in the tail so the rest is cleared
        let mut nxt = self.nxt & ((1 << off) - 1);

        let rpw = self.rpw;
        let wdt = self.wdt;

        for i in off..rpw {
            let Some(dat) = self.iter.next() else {
                // store what we currently have
                self.nxt = nxt;
                self.off = i;
                return None;
            };

            nxt |= dat << (i * wdt);

            // check quota
            self.rem -= 1;
            if self.rem == 0 {
                break;
            }
        }

        Some(nxt)
    }
}




pub struct Unpack<I> where I : Iterator<Item = u64> {
    iter: I,
    wdt: usize,
    rpw: usize,
    off: usize,
    cur: Option<u64>
}

impl<I> Iterator for Unpack<I> where I : Iterator<Item = u64> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = match self.cur {
            Some(cur) => cur,
            None => {
                let Some(nxt) = self.iter.next() else {
                    return None;
                };

                self.off = 0;
                nxt
            },
        };

        if self.wdt == 0 {
            return Some(0);
        }
        
        let msk = (1 << self.wdt) - 1;
        let off = self.off * self.wdt;

        let val = (cur >> off) & msk;
        self.off += 1;
        if self.off == self.rpw {
            self.cur = None;
        } else {
            self.cur = Some(cur);
        }

        Some(val)
    }
}




pub struct UnpackExact<I> where I : Iterator<Item = u64> {
    iter: I,
    wdt: usize,
    rpw: usize,
    off: usize,
    cur: Option<u64>,
    rem: usize
}

impl<I> Iterator for UnpackExact<I> where I : Iterator<Item = u64> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rem == 0 {
            return None;
        }

        let cur = match self.cur {
            Some(cur) => cur,

            None => {
                let Some(nxt) = self.iter.next() else {
                    return None;
                };

                self.off = 0;
                nxt
            },
        };

        if self.wdt == 0 {
            self.rem -= 1;
            return Some(0);
        }
        
        let msk = (1 << self.wdt) - 1;
        let off = self.off * self.wdt;

        let val = (cur >> off) & msk;
        self.off += 1;
        if self.off == self.rpw {
            self.cur = None;
        } else {
            self.cur = Some(cur);
        }

        self.rem -= 1;

        Some(val)
    }
}