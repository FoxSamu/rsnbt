#![allow(dead_code, unused_macros, unused_imports)]

/// # HYPER UNSAFE! USE [reinterpret_array] MACRO TO ENSURE SAFETY.
pub unsafe fn hyper_unsafe_reinterpret_array<T, U, const N: usize>(v: [T; N]) -> [U; N] where T : Copy, U : Copy {
    unsafe {
        // To avoid having Copy as bound we cannot simply cast a pointer
        // Union will do
        union Dual<T, U> {
            t: ManuallyDrop<T>,
            u: ManuallyDrop<U>,
        }

        // Set T
        let dual = Dual { t: ManuallyDrop::new(v) };

        // Obtain U
        ManuallyDrop::into_inner(dual.u)
    }
}

/// # HYPER UNSAFE! USE [reinterpret_array_ref] MACRO TO ENSURE SAFETY.
pub unsafe fn hyper_unsafe_reinterpret_array_ref<T, U, const N: usize>(v: &[T; N]) -> &[U; N] where T : Copy, U : Copy {
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
pub unsafe fn hyper_unsafe_reinterpret_slice<T, U>(v: &[T]) -> &[U] where T : Copy, U : Copy {
    unsafe {
        &*(
            v             // <- reference to slice
            as *const [T] // <- cast reference to pointer
            as *const [U] // <- cast pointer to [U] (reinterprets data)
        ) // <- get reference to dereferenced pointer to obtain &[U]
    }
}

/// # HYPER UNSAFE! USE [reinterpret_mut_slice] MACRO TO ENSURE SAFETY.
pub unsafe fn hyper_unsafe_reinterpret_mut_slice<T, U>(v: &mut [T]) -> &mut [U] where T : Copy, U : Copy {
    unsafe {
        &mut *(
            v            // <- reference to slice
            as *mut [T]  // <- cast reference to pointer
            as *mut [U]  // <- cast pointer to [U] (reinterprets data)
        ) // <- get reference to dereferenced pointer to obtain &[U]
    }
}

/// # HYPER UNSAFE! USE [reinterpret_vec] MACRO TO ENSURE SAFETY.
pub unsafe fn hyper_unsafe_reinterpret_vec<T, U>(v: Vec<T>) -> Vec<U> where T : Copy, U : Copy {
    unsafe {
        // Vec lets itself decompose and transform easily
        let (ptr, len, cap) = v.into_raw_parts();
        Vec::from_raw_parts(ptr as *mut U, len, cap)
    }
}

/// # HYPER UNSAFE! USE [reinterpret_vec_vec] MACRO TO ENSURE SAFETY.
pub unsafe fn hyper_unsafe_reinterpret_vec_vec<T, U>(v: Vec<Vec<T>>) -> Vec<Vec<U>> where T : Copy, U : Copy {
    unsafe {
        // Unfortunately we cannot safely use the raw parts of the
        // outer vec because there is no guarantee that Vec<T> and Vec<U>
        // have the same layout

        let mut result = Vec::new();

        for v in v {
            // We can apply this to the inner vecs tho
            let (p, l, c) = v.into_raw_parts();
            result.push(Vec::<U>::from_raw_parts(p as *mut U, l, c));
        }

        result
    }
}

/// Reinterprets the values of an array as a different type, with minimal copying.
/// The original type and new type must have the same size and alignment, otherwise
/// this macro will panic at compile time.
///
/// The idea is that this is substantially faster than manally converting each element,
/// since little copies are made.
macro_rules! reinterpret_array {
    ($slice:expr, $from:ty => $to:ty) => {
        unsafe {
            use $crate::util::unsafe_ops::*;

            const {
                assert!(
                    size_of::<$from>() == size_of::<$to>(),
                    concat!(stringify!($from), " and ", stringify!($to), " are not the same size!")
                );
                assert!(
                    align_of::<$from>() == align_of::<$to>(),
                    concat!(stringify!($from), " and ", stringify!($to), " are not the same alignment!")
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
/// since little copies are made.
macro_rules! reinterpret_array_ref {
    ($slice:expr, $from:ty => $to:ty) => {
        unsafe {
            use $crate::util::unsafe_ops::*;

            const {
                assert!(
                    size_of::<$from>() == size_of::<$to>(),
                    concat!(stringify!($from), " and ", stringify!($to), " are not the same size!")
                );
                assert!(
                    align_of::<$from>() == align_of::<$to>(),
                    concat!(stringify!($from), " and ", stringify!($to), " are not the same alignment!")
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
/// since little copies are made.
macro_rules! reinterpret_slice {
    ($slice:expr, $from:ty => $to:ty) => {
        unsafe {
            use $crate::util::unsafe_ops::*;

            const {
                assert!(
                    size_of::<$from>() == size_of::<$to>(),
                    concat!(stringify!($from), " and ", stringify!($to), " are not the same size!")
                );
                assert!(
                    align_of::<$from>() == align_of::<$to>(),
                    concat!(stringify!($from), " and ", stringify!($to), " are not the same alignment!")
                );
            }
            hyper_unsafe_reinterpret_slice::<$from, $to>($slice)
        }
    };
}

/// Reinterprets the values of a slice as a different type, without copying the slice.
/// The original type and new type must have the same size and alignment, otherwise
/// this macro will panic at compile time.
///
/// The idea is that this is substantially faster than manally converting each element,
/// since little copies are made.
macro_rules! reinterpret_mut_slice {
    ($slice:expr, $from:ty => $to:ty) => {
        unsafe {
            use $crate::util::unsafe_ops::*;

            const {
                assert!(
                    size_of::<$from>() == size_of::<$to>(),
                    concat!(stringify!($from), " and ", stringify!($to), " are not the same size!")
                );
                assert!(
                    align_of::<$from>() == align_of::<$to>(),
                    concat!(stringify!($from), " and ", stringify!($to), " are not the same alignment!")
                );
            }
            hyper_unsafe_reinterpret_mut_slice::<$from, $to>($slice)
        }
    };
}

/// Reinterprets the values of a vector as a different type, without copying the vector.
/// The original type and new type must have the same size and alignment, otherwise
/// this macro will panic at compile time.
///
/// The idea is that this is substantially faster than manally converting each element,
/// since little copies are made.
/// 
/// In any case, prefer reinterpreting slices over vectors, since vectors are subject to
/// way looser safety guarantees than slices in this regard.
macro_rules! reinterpret_vec {
    ($vec:expr, $from:ty => $to:ty) => {
        unsafe {
            use $crate::util::unsafe_ops::*;

            const {
                assert!(
                    size_of::<$from>() == size_of::<$to>(),
                    concat!(stringify!($from), " and ", stringify!($to), " are not the same size!")
                );
                assert!(
                    align_of::<$from>() == align_of::<$to>(),
                    concat!(stringify!($from), " and ", stringify!($to), " are not the same alignment!")
                );
            }
            hyper_unsafe_reinterpret_vec::<$from, $to>($vec)
        }
    };
}

/// Reinterprets the values of a vector as a different type, without copying the vector.
/// The original type and new type must have the same size and alignment, otherwise
/// this macro will panic at compile time.
///
/// The idea is that this is substantially faster than manally converting each element,
/// since little copies are made.
/// 
/// In any case, prefer reinterpreting slices over vectors, since vectors are subject to
/// way looser safety guarantees than slices in this regard.
macro_rules! reinterpret_vec_vec {
    ($vec:expr, $from:ty => $to:ty) => {
        unsafe {
            use $crate::util::unsafe_ops::*;

            const {
                assert!(
                    size_of::<$from>() == size_of::<$to>(),
                    concat!(stringify!($from), " and ", stringify!($to), " are not the same size!")
                );
                assert!(
                    align_of::<$from>() == align_of::<$to>(),
                    concat!(stringify!($from), " and ", stringify!($to), " are not the same alignment!")
                );
            }
            hyper_unsafe_reinterpret_vec_vec::<$from, $to>($vec)
        }
    };
}

use std::mem::ManuallyDrop;

pub(crate) use reinterpret_array;
pub(crate) use reinterpret_array_ref;
pub(crate) use reinterpret_slice;
pub(crate) use reinterpret_mut_slice;
pub(crate) use reinterpret_vec;
pub(crate) use reinterpret_vec_vec;

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
