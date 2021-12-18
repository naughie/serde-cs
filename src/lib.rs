//! `serde-cs` provides a serialization/deserialization wrapper for comma separated lists.
//!
//! # Examples
//!
//! ## Serialization
//!
//! ```rust
//! use serde_cs::vec::CS;
//! type Csu32 = CS<u32>;
//!
//! let cs: Csu32 = CS(vec![]);
//! let s = serde_json::to_string(&cs).unwrap();
//! assert_eq!(s, r#""""#);
//!
//! let cs: Csu32 = CS(vec![1]);
//! let s = serde_json::to_string(&cs).unwrap();
//! assert_eq!(s, r#""1""#);
//!
//! let cs: Csu32 = CS(vec![1, 2, 3]);
//! let s = serde_json::to_string(&cs).unwrap();
//! assert_eq!(s, r#""1,2,3""#);
//! ```
//!
//!
//! ## Deserialization
//!
//! ```rust
//! use serde_cs::vec::CS;
//! type Csu32 = CS<u32>;
//!
//! let s = r#""""#;
//! let CS(cs): Csu32 = serde_json::from_str(s).unwrap();
//! assert_eq!(cs, vec![0u32; 0]);
//!
//! let s = r#"",,,,""#;
//! let CS(cs): Csu32 = serde_json::from_str(s).unwrap();
//! assert_eq!(cs, vec![0u32; 0]);
//!
//! let s = r#"",,1,,,,,""#;
//! let CS(cs): Csu32 = serde_json::from_str(s).unwrap();
//! assert_eq!(cs, vec![1]);
//!
//! let s = r#"",,1,,,2,,,,""#;
//! let CS(cs): Csu32 = serde_json::from_str(s).unwrap();
//! assert_eq!(cs, vec![1, 2]);
//!
//! let s = r#"",,1,,,a,,,,""#;
//! let res: Result<Csu32, _> = serde_json::from_str(s);
//! assert!(res.is_err());
//! ```

pub mod array;
pub mod vec;
