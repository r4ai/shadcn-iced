//! The [Kani](https://model-checking.github.io/kani/) proofs of the
//! modules of `shadcn_iced_core` that depend on `core` alone.
//!
//! Kani compiles every dependency of a crate with its own compiler, which
//! cannot compile those of `iced`: this crate depends on nothing, and
//! includes those modules from their sources, with their proofs (in their
//! `#[cfg(kani)]` modules).
//!
//! ```text
//! cargo kani -p shadcn_iced_verify
//! ```
#[path = "../../shadcn_iced_core/src/math.rs"]
pub mod math;
