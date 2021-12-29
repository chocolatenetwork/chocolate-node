use crate::Balance;
/// Base Unit for currency
/// ====================================
/// Sinec the Max Number we can use in runtime is u32
///
/// We prepare these consts by Balance conversion/prefixing Based on SI units
/// Hence, the units of balance are:
///
/// 1 Balance - pico :10^-12
/// 1_000 Balance - nano :10^-9
/// 1_000_000 Balance - micro :10^-6
/// 1_000_000_000 Balance - milli :10^-3
/// 1_000_000_000_000 Balance - one :10^0
/// and so on. Hence, we define a base pico choc as 1Balance, and further chocs as their powers of this.
///
///
/// The balance type is general as used in our chain i.e based on 10^-12. Or pico.
pub const PICOCHOC: Balance = 1; //10^-12
/// 10^9
pub const MILLICHOC: Balance = 1_000_000_000 * PICOCHOC; // Equiv to 10^(-12+9) or milli
/// 10^12
pub const CHOC: Balance = 1_000 * MILLICHOC; // actually equiv to 1
/// 10^(12+2) === 100.{000}*4
pub const HECTOCHOC: Balance = 100 * CHOC; // actually 100. Now we start.
