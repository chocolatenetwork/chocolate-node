use crate::Balance;

// The balance type is general as used in our chain i.e based on 10^-12.
/// 10^9
pub const MILLICENTICHOC: Balance = 1_000_000_000;
/// 10^12
pub const CENTICHOC: Balance = 1_000 * MILLICENTICHOC;
/// 10^(12+2) === 100.{000}*4
pub const CHOC: Balance = 100 * CENTICHOC;