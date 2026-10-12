#![allow(dead_code, unused_macros, unused_imports)]

/// # HYPER UNSAFE! USE [reinterpret_array] MACRO TO ENSURE SAFETY.
pub unsafe fn hyper_unsafe_reinterpret_array<T, U, const N: usize>(v: [T; N]) -> [U; N] {
    unsafe {
        // To avoid having Copy as bound we cannot simply cast a pointer
        // Union will do
        union Dual<T, U> {
            t: ManuallyDrop<T>,
            u: ManuallyDrop<U>,
        }

        // Set T
        let dual = Dual {
            t: ManuallyDrop::new(v),
        };

        // Obtain U
        ManuallyDrop::into_inner(dual.u)
    }
}

/// # HYPER UNSAFE! USE [reinterpret_array_ref] MACRO TO ENSURE SAFETY.
pub unsafe fn hyper_unsafe_reinterpret_array_ref<T, U, const N: usize>(v: &[T; N]) -> &[U; N] {
    unsafe {
        &*(
            v                  // <- reference to slice
            as *const [T; N]   // <- cast reference to pointer
            as *const [U; N]
            // <- cast pointer to [U] (reinterprets data)
        ) // <- get reference to dereferenced pointer to obtain &[U]
    }
}

/// # HYPER UNSAFE! USE [reinterpret_slice] MACRO TO ENSURE SAFETY.
pub unsafe fn hyper_unsafe_reinterpret_slice<T, U>(v: &[T]) -> &[U] {
    unsafe {
        &*(
            v                  // <- reference to slice
            as *const [T]      // <- cast reference to pointer
            as *const [U]
            // <- cast pointer to [U] (reinterprets data)
        ) // <- get reference to dereferenced pointer to obtain &[U]
    }
}

/// # HYPER UNSAFE! USE [reinterpret_vec] MACRO TO ENSURE SAFETY.
pub unsafe fn hyper_unsafe_reinterpret_vec<T, U>(v: Vec<T>) -> Vec<U> {
    unsafe {
        // Vec lets itself decompose and transform easily
        let (ptr, len, cap) = v.into_raw_parts();
        Vec::from_raw_parts(ptr as *mut U, len, cap)
    }
}

/// Reinterprets the values of an array as a different type, with minimal copying.
/// The original type and new type must have the same size and alignment, otherwise
/// this macro will panic at compile time.
///
/// The idea is that this is substantially faster than manally converting each element,
/// since little copies are made.
///
/// ```
/// let i8s: [i8; 3] = [3, -1, 100];
/// let u8s = reinterpret_array!(i8s, i8 => u8; 3);
///
/// assert_eq!([3, 255, 100], u8s);
/// ```
macro_rules! reinterpret_array {
    ($slice:expr, $from:ty => $to:ty) => {
        unsafe {
            use $crate::unsafe_ops::*;

            const {
                assert!(
                    size_of::<$from>() == size_of::<$to>(),
                    concat!(stringify!($from), " and ", stringify!($to), " are not the same size!")
                );
                assert!(
                    align_of::<$from>() == align_of::<$to>(),
                    concat!(
                        stringify!($from),
                        " and ",
                        stringify!($to),
                        " are not the same alignment!"
                    )
                );
            }
            hyper_unsafe_reinterpret_array::<$from, $to, _>($slice)
        }
    };
}

/// Reinterprets the values of an array reference as a different type, without copying
/// the array.
/// The original type and new type must have the same size and alignment, otherwise
/// this macro will panic at compile time.
///
/// The idea is that this is substantially faster than manally converting each element,
/// since no copies are made.
///
/// ```
/// let i8s: [i8; 3] = [3, -1, 100];
/// let u8s = reinterpret_array_ref!(&i8s, i8 => u8; 3);
///
/// assert_eq!(&[3, 255, 100], u8s);
/// ```
macro_rules! reinterpret_array_ref {
    ($slice:expr, $from:ty => $to:ty) => {
        unsafe {
            use $crate::unsafe_ops::*;

            const {
                assert!(
                    size_of::<$from>() == size_of::<$to>(),
                    concat!(stringify!($from), " and ", stringify!($to), " are not the same size!")
                );
                assert!(
                    align_of::<$from>() == align_of::<$to>(),
                    concat!(
                        stringify!($from),
                        " and ",
                        stringify!($to),
                        " are not the same alignment!"
                    )
                );
            }
            hyper_unsafe_reinterpret_array_ref::<$from, $to, _>($slice)
        }
    };
}

/// Reinterprets the values of a slice as a different type, without copying the slice.
/// The original type and new type must have the same size and alignment, otherwise
/// this macro will panic at compile time.
///
/// The idea is that this is substantially faster than manally converting each element,
/// since no copies are made.
///
/// ```
/// let i8s: [i8; 3] = [3, -1, 100];
/// let u8s = reinterpret_slice!(&i8s, i8 => u8);
///
/// assert_eq!(&[3, 255, 100], u8s);
/// ```
macro_rules! reinterpret_slice {
    ($slice:expr, $from:ty => $to:ty) => {
        unsafe {
            use $crate::unsafe_ops::*;

            const {
                assert!(
                    size_of::<$from>() == size_of::<$to>(),
                    concat!(stringify!($from), " and ", stringify!($to), " are not the same size!")
                );
                assert!(
                    align_of::<$from>() == align_of::<$to>(),
                    concat!(
                        stringify!($from),
                        " and ",
                        stringify!($to),
                        " are not the same alignment!"
                    )
                );
            }
            hyper_unsafe_reinterpret_slice::<$from, $to>($slice)
        }
    };
}

/// Reinterprets the values of a vector as a different type, without copying the vector.
/// The original type and new type must have the same size and alignment, otherwise
/// this macro will panic at compile time.
///
/// The idea is that this is substantially faster than manally converting each element,
/// since no copies are made.
///
/// ```
/// let i8s: Vec<i8> = [3, -1, 100];
/// let u8s = reinterpret_vec!(i8s, i8 => u8);
///
/// assert_eq!(vec![3, 255, 100], u8s);
/// ```
macro_rules! reinterpret_vec {
    ($vec:expr, $from:ty => $to:ty) => {
        unsafe {
            use $crate::unsafe_ops::*;

            const {
                assert!(
                    size_of::<$from>() == size_of::<$to>(),
                    concat!(stringify!($from), " and ", stringify!($to), " are not the same size!")
                );
                assert!(
                    align_of::<$from>() == align_of::<$to>(),
                    concat!(
                        stringify!($from),
                        " and ",
                        stringify!($to),
                        " are not the same alignment!"
                    )
                );
            }
            hyper_unsafe_reinterpret_vec::<$from, $to>($vec)
        }
    };
}

use std::mem::ManuallyDrop;

pub(super) use reinterpret_array;
pub(super) use reinterpret_array_ref;
pub(super) use reinterpret_slice;
pub(super) use reinterpret_vec;

#[cfg(test)]
mod test {
    #[test]
    fn test_reinterpret_array() {
        let i8s: [i8; 3] = [3, -1, 100];
        let u8s = reinterpret_array!(i8s, i8 => u8);

        assert_eq!([3, 255, 100], u8s);
    }

    #[test]
    fn test_reinterpret_array_ref() {
        let i8s: [i8; 3] = [3, -1, 100];
        let u8s = reinterpret_array_ref!(&i8s, i8 => u8);

        assert_eq!(&[3, 255, 100], u8s);
    }

    #[test]
    fn test_reinterpret_slice() {
        let i8s: [i8; 3] = [3, -1, 100];
        let u8s = reinterpret_slice!(&i8s, i8 => u8);

        assert_eq!(&[3, 255, 100], u8s);
    }

    #[test]
    fn test_reinterpret_vec() {
        let i8s: Vec<i8> = vec![3, -1, 100];
        let u8s = reinterpret_vec!(i8s, i8 => u8);

        assert_eq!(vec![3, 255, 100], u8s);
    }
}
