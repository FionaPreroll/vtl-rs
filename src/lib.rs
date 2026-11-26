//! # vtl-rs
//!
//! An interpreter for VTL-2, the *Very Tiny Language* that Frank McCoy and
//! Gary Shannon wrote in 1977 for the Altair 680 and that later ran on the
//! Altair 8800 and the 6502 (VTL02). It follows the documented behaviour of
//! the original: 16-bit unsigned numbers, expressions strictly from left to
//! right, programs made of numbered lines and the system variables `#`, `!`,
//! `?`, `$`, `%`, `&`, `*` and `'`. The VTL02 extensions are on by default
//! and can be switched off ([`syntax::Syntax::extended`]).
//!
//! The crate has no dependencies; the command line program `vtl-rs` (feature
//! `cli`) and the web version build on it.
//!
//! ```
//! use vtl_rs::machine::{Config, Machine, Status};
//!
//! let mut vtl = Machine::new(Config { prompt: false, ..Config::default() });
//! vtl.enter("10 I=1");
//! vtl.enter("20 ?=I*I");
//! vtl.enter("25 ?=\" \";");
//! vtl.enter("30 I=I+1");
//! vtl.enter("40 #=I<6*20");
//! let mut status = vtl.enter("#=1");
//! while status == Status::Running {
//!     status = vtl.run(1000);
//! }
//! assert_eq!(vtl.take_output(), b"1 4 9 16 25 ");
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod machine;
pub mod syntax;

pub use machine::{Config, Machine, Status};
