//! A collection of primitives used throughout chocolate

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
/// Module of constants describing the units of the fungible token system used.
pub mod currency;
/// Module of constants describing the units of time used on-chain for ease.
pub mod time;
pub use crate::{currency::*, time::*};

/// Balance of an account.
pub type Balance = u128;
/// An index to a block.
pub type BlockNumber = u32;
