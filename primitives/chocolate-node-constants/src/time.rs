use crate::BlockNumber;
/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 6000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
/// Equivalent to the milliseconds per block.
/// Time is measured by number of blocks for consensus.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;
/// An approximate minute
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
/// An approximate Hour in block time
pub const HOURS: BlockNumber = MINUTES * 60;
/// An approximate hour in block time
pub const DAYS: BlockNumber = HOURS * 24;
// Interesting use of block number which everyone agrees on for time.