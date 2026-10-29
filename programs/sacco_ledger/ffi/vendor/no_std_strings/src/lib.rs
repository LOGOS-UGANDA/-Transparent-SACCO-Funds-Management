//! **`no_std`** version of crate **[fixedstr](https://crates.io/crates/fixedstr/)**:
//! strings of fixed maximum lengths that can be copied and
//! stack-allocated using const generics.

//! The types provided by this crate are **[zstr]** and **`tstr`**.  However,
//! tstr is not directly exported and can only be directly referenced through
//! the type aliases [str4]-[str256].  Each `zstr<N>` represents a
//! zero-terminated string of exactly N bytes, accommodating all strings of
//! lengths up to N-1 bytes.  A `tstr<N>` can likewise hold all strings of
//! up to N-1 bytes, but it stores the length of the string in the first byte.
//! Thus the const generic parameter `N` cannot exceed 256.  Since there is
//! still no stable way to contrain N at compile time, the tstr type can only
//! be referenced using the aliases.  With few exceptions the tstr type
//! implement the same functions and traits as [zstr].
//!  
//! Compared to their counterparts in
//! [fixedstr](https://docs.rs/fixedstr/latest/fixedstr/), some functions
//! were omitted to accommodate the `#![no_std]` requirement.
//!
//! Optional serde serialization support is enabled by `--features serde`.


#![no_std]

#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(dead_code)]

mod zero_terminated;
pub use zero_terminated::*;

mod tiny_internal;
use tiny_internal::*;

/// Types for small strings that use a more efficient representation
/// underneath.  A str8 can hold a string of up to 7 bytes (7 ascii chars).
/// The same functions for [zstr] are provided for these types
/// so the documentation for the other types also applies.
/// The size of str8 is 8 bytes.
///
/// Example:
/// ```
///  use no_std_strings::str8;
///  let mut s = str8::from("aλc");
///  assert_eq!(s.capacity(),7);
///  assert_eq!(s.push("1234567"), "4567");
///  assert_eq!(s,"aλc123");
///  assert_eq!(s.charlen(), 6);
///  assert_eq!(s.len(), 7);  
/// ```
/// All functions and traits for these types mirror those of [zstr]. 
pub type str8 = tstr<8>;
/// A str16 can hold a string of up to 15 bytes. See docs for [zstr].
/// The size of str16 is 16 bytes, which is the same as for &str on 64bit
/// systems.
pub type str16 = tstr<16>;
/// A str32 can hold a string of up to 31 bytes. See docs for [zstr]
pub type str32 = tstr<32>;
/// A str64 can hold a string of up to 63 bytes. See docs for [zstr]
pub type str64 = tstr<64>;
/// A str28 can hold a string of up to 127 bytes. See docs for [zstr]
pub type str128 = tstr<128>;
/// Each type strN is represented underneath by a `[u8;N]` with N<=256.
/// The first byte of the array always holds the length of the string.
/// Each such type can hold a string of up to N-1 bytes, with max size=255.
/// <br>
/// In addition, the str4-str128 types implement [core::ops::Add], allowing for
/// string concatenation of strings of the same type.  For example,
/// two str8 strings will always concatenate to str16, and similarly for
/// all other strN types up to str128.
///```
///  use no_std_strings::str8;
///  let c1 = str8::from("abcd");
///  let c2 = str8::from("xyz");
///  let c3 = c1 + c2;
///  assert_eq!(c3,"abcdxyz");
///  assert_eq!(c3.capacity(),15);
///```
pub type str256 = tstr<256>;

/// strings of up to three 8-bit chars, good enough to represent abbreviations
/// such as those for states and airports. Each str<4> is exactly 32 bits.
pub type str4 = tstr<4>;
pub type str12 = tstr<12>;
pub type str24 = tstr<24>;
pub type str48 = tstr<48>;
pub type str96 = tstr<96>;
pub type str192 = tstr<192>;




#[macro_export]
/// creates a formated string of given type (by implementing [core::fmt::Write]):
/// ```ignore
///    let s = str_format!(str8,"abc{}{}{}",1,2,3);
/// ```
/// will truncate if capacity exceeded, without warning.
macro_rules! str_format {
  ($ty_size:ty, $($args:tt)*) => {
     {use core::fmt::Write;
     let mut fstr0 = <$ty_size>::new();
     let res=write!(&mut fstr0, $($args)*);
     fstr0}
  };
}

#[macro_export]
/// version of [str_format]! that returns an Option of the given type.
/// ```ignore
///  let s = try_format!(str32,"abcdefg{}","hijklmnop").unwrap();
///  let s2 = try_format!(str8,"abcdefg{}","hijklmnop");
///  assert!(s2.is_none());
/// ```
macro_rules! try_format {
  ($ty_size:ty, $($args:tt)*) => {
     {use core::fmt::Write;
     let mut fstr0 = <$ty_size>::new();
     let result = write!(&mut fstr0, $($args)*);
     if result.is_ok() {Some(fstr0)} else {None}}
  };
}


#[cfg(feature="serde")]
mod serde_support {
    use serde::{Serialize, Deserialize, Serializer, Deserializer, de::Visitor};
    use super::*;
    macro_rules! generate_impl {
        ($ty: ident, $visitor: ident) => {
            impl<const N: usize> Serialize for $ty<N> {
                fn serialize<S: Serializer>(&self, serializer:S) -> Result<S::Ok, S::Error> {
                    serializer.serialize_str(self.as_str())
                }
            }
            impl<'de, const N: usize> Deserialize<'de> for $ty<N> {
                fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                    deserializer.deserialize_str($visitor)
                }
            }
            struct $visitor<const N: usize>;
            impl<'de, const N: usize> Visitor<'de> for $visitor<N> {
                type Value = $ty<N>;
                fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.write_str("a string")
                }
                fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
                    $ty::try_make(s).map_err(|_| E::custom("string too long"))
                }
            }
        }
    }
    generate_impl!(zstr, ZstrVisitor);
    generate_impl!(tstr, TstrVisitor);
}



/*
fn tests() {
  let a:str8 = str8::from("abcdef"); //a str8 can hold up to 7 bytes
  let a2 = a;  // copied, not moved
  let ab = a.substr(1,5);  // copies substring to new string
  assert_eq!(ab, "bcde");  // can compare for equality with &str
  assert_eq!(ab.len(),4);
  assert_eq!(&a[..3], "abc"); // impls Index for Range types
  assert!(a<ab); // and Ord, Hash, Debug, Display, Eq, other common traits
  let astr:&str = a.to_str(); // convert to &str (zero copy)
  let azstr:zstr<16> = zstr::from(a); // so is zstr
  let a32:str32 = a.resize(); // same kind of string but with 31-byte capacity  
  let mut u = str8::from("aλb"); //unicode support
  assert_eq!(u.nth(1), Some('λ'));  // get nth character
  assert_eq!(u.nth_ascii(3), 'b');  // get nth byte as ascii character
  assert!(u.set(1,'μ'));  // changes a character of the same character class
  assert!(!u.set(1,'c')); // .set returns false on failure
  assert!(u.set(2,'c'));
  assert_eq!(u, "aμc");
  assert_eq!(u.len(),4);  // length in bytes
  assert_eq!(u.charlen(),3);  // length in chars
  let mut ac:str16 = a.reallocate().unwrap(); //copies to larger capacity type
  let remainder = ac.push("ghijklmnopq"); //append up to capacity, returns remainder
  assert_eq!(ac.len(),15);
  assert_eq!(remainder, "pq");
  ac.truncate(9);  // keep first 9 chars
  assert_eq!(&ac,"abcdefghi");
  let (upper,lower) = (str8::make("ABC"), str8::make("abc"));
  assert_eq!(upper, lower.to_ascii_upper()); // no owned String needed

  let c1 = str8::from("abcd"); // string concatenation with + for strN types  
  let c2 = str8::from("xyz");
  let c3 = c1 + c2;           
  assert_eq!(c3,"abcdxyz");
  assert_eq!(c3.capacity(),15);  // type of c3 is str16

  let c4 = str_format!(str16,"abc {}{}{}",1,2,3); // impls std::fmt::Write
  assert_eq!(c4,"abc 123");  //str_format! truncates if capacity exceeded
  let c5 = try_format!(str8,"abcdef{}","ghijklmn");
  assert!(c5.is_none());  // try_format! returns None if capacity exceeded
}//tests

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_works() {
      tests();
    }
}
*/
