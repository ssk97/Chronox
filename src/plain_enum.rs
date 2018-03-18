//! This crate offers some tools to deal with static enums. It offers a way to declare a simple
//! enum, which then offers e.g. `values()` which can be used to iterate over the values of the enum.
//! In addition, it offers a type `EnumMap` which is an array-backed map from enum values to some type.
//!
//! It offers a macro `plain_enum_mod` which declares an own module which contains a simple enum
//! and the associated functionality:
//!
//! ```
//! mod examples_not_to_be_used_by_clients {
//!     #[macro_use]
//!     use plain_enum::*;
//!     plain_enum_mod!{example_mod_name, ExampleEnum {
//!         V1,
//!         V2,
//!         SomeOtherValue,
//!         LastValue, // note trailing comma
//!     }}
//!
//!     fn do_some_stuff() {
//!         let map = ExampleEnum::map_from_fn(|example| // create a map from ExampleEnum to usize
//!             example.to_usize() + 1                   // enum values convertible to usize
//!         );
//!         for ex in ExampleEnum::values() {            // iterating over the enum's values
//!             assert_eq!(map[ex], ex.to_usize() + 1);
//!         }
//!     }
//! }
//! ```
//!
//! Internally, the macro generates a simple enum whose numeric values start counting at 0.

#[macro_use]
mod plain_enum {
    #[macro_export]
    macro_rules! enum_seq_len {
        ($n: expr, $enumval: ident,) => ($n);
        ($n: expr, $enumval: ident, $($enumvals: ident,)*) => (enum_seq_len!(($n + 1), $($enumvals,)*));
    }

    use std::iter;
    use std::ops;

    /// This trait is implemented by enums declared via the `plain_enum_mod` macro.
    /// Do not implement it yourself, but use this macro.
    pub trait TPlainEnum : Sized {
        /// Arity, i.e. the smallest `usize` not representable by the enum.
        const SIZE : usize;
        /// Checks whether `u` is the numerical representation of a valid enum value.
        fn valid_usize(u: usize) -> bool;
        /// Converts `u` to the associated enum value. `assert`s that `u` is a valid value for the enum.
        fn from_usize(u: usize) -> Self;
        /// Converts `u` to the associated enum value. if `u` is a valid value for the enum.
        fn checked_from_usize(u: usize) -> Option<Self>;
        /// Converts `u` to the associated enum value, but wraps `u` it before conversion (i.e. it
        /// applies the modulo operation with a modulus equal to the arity of the enum before converting).
        fn wrapped_from_usize(u: usize) -> Self;
        /// Computes the difference between two enum values, wrapping around if necessary.
        fn wrapped_difference(self, e_other: Self) -> usize;
        /// Converts the enum to its numerical representation.
        fn to_usize(self) -> usize;
        /// Returns an iterator over the enum's values.
        fn values() -> iter::Map<ops::Range<usize>, fn(usize) -> Self> {
            (0..Self::SIZE)
                .map(Self::from_usize)
        }
        /// Adds a number to the enum, wrapping.
        fn wrapping_add(self, n_offset: usize) -> Self;
    }

    /// Trait used to associated enum with EnumMap.
    pub trait TEnumMapType<T> : TPlainEnum {
        type MapType;
    }

    #[allow(dead_code)]
    // TODO rust: trait bounds are not (yet) enforced in type definitions (rust 1.16, 20170408)
    pub type EnumMap<PlainEnum, T> = <PlainEnum as TEnumMapType<T>>::MapType;

    #[macro_export]
    macro_rules! acc_arr {
        ($func: ident, [$($acc: expr,)*], []) => {
            [$($acc,)*]
        };
        ($func: ident, [$($acc: expr,)*], [$enumval: ident, $($enumvals: ident,)*]) => {
            acc_arr!($func, [$($acc,)* $func($enumval),], [$($enumvals,)*])
        };
    }

    #[macro_export]
    macro_rules! plain_enum_mod {
        ($modname: ident, derive($($derives:ident, )*), map_derive($($mapderives:ident, )*), $enumname: ident {
            $($enumvals: ident,)*
        } ) => {
            mod $modname {
                use plain_enum::*;
                use std::slice;
                #[repr(usize)]
                #[derive(PartialEq, Eq, Debug, Copy, Clone, PartialOrd, Ord, $($derives,)*)]
                pub enum $enumname {
                    $($enumvals,)*
                }

                impl TPlainEnum for $enumname {

                    const SIZE : usize = enum_seq_len!(1, $($enumvals,)*);

                    fn valid_usize(u: usize) -> bool {
                        u < Self::SIZE
                    }
                    fn from_usize(u: usize) -> Self {
                        use std::mem;
                        debug_assert!(Self::valid_usize(u));
                        unsafe{mem::transmute(u)}
                    }
                    fn checked_from_usize(u: usize) -> Option<Self> {
                        if Self::valid_usize(u) {
                            Some(Self::from_usize(u))
                        } else {
                            None
                        }
                    }
                    fn wrapped_from_usize(u: usize) -> Self {
                        Self::from_usize(u % Self::SIZE)
                    }
                    fn wrapped_difference(self, e_other: Self) -> usize {
                        (self.to_usize() + Self::SIZE - e_other.to_usize()) % Self::SIZE
                    }
                    fn to_usize(self) -> usize {
                        self as usize
                    }
                    fn wrapping_add(self, n_offset: usize) -> Self {
                        Self::from_usize((self.to_usize() + n_offset) % Self::SIZE)
                    }
                }

                impl $enumname {
                    #[allow(dead_code)]
                    /// Creates a enum map from enum values to a type, determined by `func`.
                    /// The map will contain the results of applying `func` to each enum value.
                    pub fn map_from_fn<F, T>(mut func: F) -> Map<T>
                        where F: FnMut($enumname) -> T,
                    {
                        use self::$enumname::*;
                        Map::from_raw(acc_arr!(func, [], [$($enumvals,)*]))
                    }
                    /// Creates a enum map from a raw array.
                    #[allow(dead_code)]
                    pub fn map_from_raw<T>(at: [T; Self::SIZE]) -> Map<T> {
                        Map::from_raw(at)
                    }
                }

                impl<T> TEnumMapType<T> for $enumname {
                    type MapType = Map<T>;
                }

                use std::ops::{Index, IndexMut};
                #[derive(Clone, PartialEq, Eq, Debug, $($mapderives,)*)]
                pub struct Map<T> {
                    m_at : [T; $enumname::SIZE],
                }
                impl<T> Index<$enumname> for Map<T> {
                    type Output = T;
                    fn index(&self, e : $enumname) -> &T {
                        &self.m_at[e.to_usize()]
                    }
                }
                impl<T> IndexMut<$enumname> for Map<T> {
                    fn index_mut(&mut self, e : $enumname) -> &mut Self::Output {
                        &mut self.m_at[e.to_usize()]
                    }
                }
                impl<T> Map<T> {
                    #[allow(dead_code)]
                    pub fn iter(&self) -> slice::Iter<T> {
                        self.m_at.iter()
                    }
                    pub fn from_raw(at: [T; $enumname::SIZE]) -> Map<T> {
                        Map {
                            m_at : at,
                        }
                    }
                }
                impl<T: Copy> Map<T> {
                    #[allow(dead_code)]
                    pub fn new(val: T) -> Map<T> {
                        Map {m_at: [val;$enumname::SIZE]}
                    }
                }
            }
            pub use self::$modname::$enumname;
        };
        ($modname: ident, $enumname: ident {
            $($enumvals: ident,)*
        } ) => {
            plain_enum_mod!($modname, derive(), map_derive(), $enumname { $($enumvals,)* });
        };
    }
}
pub use self::plain_enum::TPlainEnum;
pub use self::plain_enum::TEnumMapType;
pub use self::plain_enum::EnumMap;