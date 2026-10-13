#![allow(dead_code)]

use core::slice;
use std::alloc::Layout;
use std::alloc::alloc;
use std::alloc::dealloc;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::ops::Index;
use std::ops::IndexMut;
use std::ptr::copy_nonoverlapping;
use std::ptr::null_mut;
use std::ptr::read;

/// A ring buffer.
///
/// A ring buffer with a maximum capacity of `N` remembers the last `N` inserted elements in the
/// order they were inserted. When inserting an `N+1`th element, the first element will be moved
/// out of the ring buffer. It therefore acts like a fixed-capacity first-in first-out data
/// structure.
///
/// [`RingBuf`] keeps elements in a coherent block of memory, meaning that if some elements in the
/// ring buffer appear in a certain order, they are necessarily next to eachother in memory. Simply
/// put, the values in a ring buffer can always be returned in one coherent slice.
pub struct RingBuf<T> {
    buf: *mut T,
    layout: Layout,
    size: usize,
    pos: usize,
    len: usize,
}

impl<T> RingBuf<T> {
    /*

    The ring buffer is implemented using a buffer of 2N elements, N being the maximum capacity.
    A window of at most N elements shifts over this buffer. Values are considered initialised and
    need to be dropped IF AND ONLY IF they are within the current window.

    If the window position P reaches the capacity N, then the window is at the end of the buffer,
    and to insert the next element, the elements in the window are copied back to the start of the
    buffer. Only then a new element can be added.

    The reason we implement the ring buffer while keeping half the memory unused is so that we can
    keep the data coherent in memory, allowing us to return a slice with all the data coherently
    packed without copying the data. Since the copy only happens every N insertions and takes O(N),
    the average insertion speed is O(1).

    */

    /// Creates a new [`RingBuf`] with a maximum capacity of `size` elements.
    pub fn new(size: usize) -> Self {
        let layout = Layout::array::<T>(size * 2).unwrap();

        Self {
            buf: unsafe {
                if size == 0 {
                    null_mut()
                } else {
                    alloc(layout) as *mut T
                }
            },
            layout,
            size,
            pos: 0,
            len: 0,
        }
    }

    fn move_in(&mut self, offset: usize, value: T) {
        unsafe { *self.address(offset) = value }
    }

    fn move_out(&mut self, offset: usize) -> T {
        unsafe { read(self.address(offset)) }
    }

    fn borrow(&self, offset: usize) -> &mut T {
        unsafe { &mut *self.address(offset) }
    }

    fn address(&self, offset: usize) -> *mut T {
        unsafe { self.buf.offset(offset as isize) }
    }

    /// Inserts a new element into the ring buffer. If the ring buffer is at maximum capacity, the
    /// oldest element in the buffer will be removed and returned.
    ///
    /// More specifically, the oldest element *after* the insertion is returned. Thus, in the case
    /// of a zero-capacity buffer, the new element is simply immediately returned as it is, in that
    /// exact moment the oldest and only element in the buffer.
    ///
    /// If the buffer is not full, no element is removed and this method returns [`None`].
    ///
    /// # Time complexity
    /// Let `N` be the capacity of the buffer. This method must sometimes copy up to `N` elements,
    /// since it may have reached the end of the internal buffer. This happens at most every `N`
    /// insertions. Therefore, one insertion may take up to `O(N)` but the average time complexity
    /// is `O(1)`.
    pub fn put(&mut self, val: T) -> Option<T> {
        if self.size == 0 {
            return Some(val);
        }

        if self.len == 0 {
            // we are empty, move window to the start in the hope to reduce
            // the amount of copies we do
            self.pos = 0;
        }

        if self.pos + self.len == self.size * 2 {
            // Exceeded buffer size: copy everything to start of buffer

            unsafe {
                let addr = self.buf.offset(self.pos as isize);
                copy_nonoverlapping(addr, self.buf, self.len);

                self.pos = 0;
            }
        }

        if self.len < self.size {
            // Capacity is not reached.

            self.move_in(self.len, val);

            self.len += 1;

            // Not full, all elements stay in buffer
            None
        } else {
            // Capacity is reached, the last inserted elements drops.

            self.move_in(self.len + self.pos, val);

            let prev = self.move_out(self.pos);

            self.pos += 1;

            // Not full, all elements stay in buffer
            Some(prev)
        }
    }

    /// Removes and returns the oldest element in the buffer. This does not insert a new element,
    /// and the [length][RingBuf::len] will shrink.
    ///
    /// If the buffer is empty, this returns [`None`].
    ///
    /// # Time complexity
    /// Removing an element is always `O(1)` time complexity.
    pub fn poll(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        let v = self.move_out(self.pos);
        self.pos += 1;
        self.len -= 1;
        Some(v)
    }

    /// Returns a coherent slice of the elements currently in this buffer. The slice is immutable
    /// and contains at most [`capacity()`][RingBuf::capacity] elements.
    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.buf.offset(self.pos as isize), self.len) }
    }

    /// Returns a coherent slice of the elements currently in this buffer. The slice is mutable
    /// and contains at most [`capacity()`][RingBuf::capacity] elements.
    pub fn as_slice_mut(&self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.buf.offset(self.pos as isize), self.len) }
    }

    /// Returns the length of the ring buffer. This is the amount of elements currently in the
    /// buffer. Logically, the returned length is never more than [`capacity()`][RingBuf::capacity].
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the capacity of the ring buffer, i.e. the maximum number of elements the buffer can
    /// store. This value never changes over the lifetime of a single ring buffer.
    pub fn capacity(&self) -> usize {
        self.size
    }

    /// Returns whether this ring buffer is empty, i.e. whether it contains no elements. This is
    /// analogous to checking if [`RingBuf::len`] returns `0`.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns whether this ring buffer is full, i.e. whether it is at maximum capacity. This is
    /// analogous to checking if [`RingBuf::len`] returns the same value as [`RingBuf::capacity`].
    pub fn is_full(&self) -> bool {
        self.len == self.size
    }

    /// Returns an [`Iter`], which iterates over all the elements in this ring buffer without moving
    /// them. The iterator returns immutable references to all the values in this ring buffer, in
    /// the order of insertion.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter { buf: self, idx: 0 }
    }

    /// Returns an [`IterMut`], which iterates over all the elements in this ring buffer without
    /// moving them. The iterator returns mutable references to all the values in this ring buffer,
    /// in the order of insertion.
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            buf: self as *mut Self,
            idx: 0,
            _phantom: &PhantomData,
        }
    }

    /// Removes and drops all elements in the ring buffer.
    pub fn clear(&mut self) {
        for i in self.pos..(self.pos + self.len) {
            // Calling drop is implied by the compiler but let's just be
            // explicit here: we own this value and we correctly dispose
            // of it.
            drop(self.move_out(i));
        }

        self.pos = 0;
        self.len = 0;
    }
}

/// Allows indexing of a [`RingBuf`].
impl<T> Index<usize> for RingBuf<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!(
                "index out of bounds: index was {} but buffer only has {} elements",
                index, self.len
            );
        }

        self.borrow(self.pos + index)
    }
}

/// Allows mutable indexing of a [`RingBuf`].
impl<T> IndexMut<usize> for RingBuf<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.len {
            panic!(
                "index out of bounds: index was {} but buffer only has {} elements",
                index, self.len
            );
        }

        self.borrow(self.pos + index)
    }
}

/// [`Drop`] implementation for [`RingBuf`]. This drops all the elements in the ring buffer and then
/// deallocates the internal memory buffer.
impl<T> Drop for RingBuf<T> {
    fn drop(&mut self) {
        unsafe {
            // Drop elements in buffer, clear does this
            self.clear();

            // Die peacefully
            dealloc(self.buf as *mut u8, self.layout);
        }
    }
}

/// [`Clone`] implementation for [`RingBuf`]s whose elements implement [`Clone`]. This creates an
/// identical [`RingBuf`], though its internals may differ.
impl<T> Clone for RingBuf<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut buf = Self::new(self.size);
        for i in self.pos..(self.pos + self.len) {
            buf.move_in(i - self.pos, self.borrow(i).clone());
        }
        buf.len = self.len;
        buf
    }
}

/// [`IntoIterator`] implementation for [`RingBuf`]. This implementation allows ring buffers to be
/// moved into iterators, to move out their elements.
impl<T> IntoIterator for RingBuf<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { buf: self }
    }
}

/// [`Debug`] implementation for [`RingBuf`]s whose elements implement [`Debug`]. Prints the
/// elements much like a [`Vec`]: `[1, 2, 3, ...]`.
impl<T> Debug for RingBuf<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

/// [`PartialEq`] implementation for [`RingBuf`]s whose elements implement [`PartialEq`].
impl<T> PartialEq for RingBuf<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if other.len != self.len {
            return false;
        }

        for i in 0..self.len {
            if other[i] != self[i] {
                return false;
            }
        }

        true
    }
}

/// [`Eq`] implementation for [`RingBuf`]s whose elements implement [`Eq`].
impl<T> Eq for RingBuf<T>
where
    T: Eq,
{
    // N/A
}

/// [`PartialOrd`] implementation for [`RingBuf`]s whose elements implement [`PartialOrd`].
impl<T> PartialOrd for RingBuf<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        for i in 0..self.len.min(other.len) {
            match self[i].partial_cmp(&other[i]) {
                Some(Ordering::Equal) => continue,
                o => return o,
            }
        }

        Some(Ordering::Equal)
    }
}

/// [`Ord`] implementation for [`RingBuf`]s whose elements implement [`Ord`].
impl<T> Ord for RingBuf<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        for i in 0..self.len.min(other.len) {
            match self[i].cmp(&other[i]) {
                Ordering::Equal => continue,
                o => return o,
            }
        }

        Ordering::Equal
    }
}

pub struct Iter<'a, T> {
    buf: &'a RingBuf<T>,
    idx: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.buf.len {
            let idx = self.idx;
            self.idx += 1;
            Some(&self.buf[idx])
        } else {
            None
        }
    }
}

pub struct IterMut<'a, T> {
    buf: *mut RingBuf<T>,
    idx: usize,
    _phantom: &'a PhantomData<T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<&'a mut T> {
        let buf = unsafe { &mut *self.buf };

        if self.idx < buf.len {
            let idx = self.idx;
            self.idx += 1;

            Some(&mut buf[idx])
        } else {
            None
        }
    }
}

pub struct IntoIter<T> {
    buf: RingBuf<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.buf.poll()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test1() {
        let mut buf = RingBuf::new(3);

        buf.put(1);
        buf.put(2);

        assert_eq!(vec![1, 2], Vec::from_iter(buf.iter().copied()));

        buf.put(3);
        buf.put(4);

        assert_eq!(vec![2, 3, 4], Vec::from_iter(buf.iter().copied()));

        buf.put(5);
        buf.put(6);
        buf.put(7);

        assert_eq!(vec![5, 6, 7], Vec::from_iter(buf.iter().copied()));
    }
}
