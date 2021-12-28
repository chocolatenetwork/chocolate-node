#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchResult;
use frame_system::Config;
use sp_std::vec::Vec;
/// type alias for text
pub type TextAl = Vec<u8>;
/// A simple u32
pub type ProjectID = u32;
/// Index for reviews , use to link to project
pub type ReviewID = u64;
use codec::{Decode, Encode};
#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Review<UserID> {
	pub proposal_status: ProposalStatus,
	pub user_id: UserID,
	pub content: Vec<u8>,
	pub project_id: ProjectID,
	/// A snapshot of the user's rank at the time of review
	pub point_snapshot: u32,
}

/// The metadata of a project.
type MetaData = Vec<u8>;

#[cfg(feature = "std")]
pub use serde::{Deserialize, Serialize};

/// The status of the proposal
#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
pub enum Status {
	///Proposal created
	Proposed,
	/// Proposal accepted
	Accepted,
	/// Proposal rejected
	Rejected,
}
/// Reason for the current status - Required for rejected proposal.
#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
pub enum Reason {
	/// Custom reason to encapsulate further things like marketCap and other details
	Other(Vec<u8>),
	/// Negative lenient - base conditions for project missing or review lacking detail
	InsufficientMetaData,
	/// Negative harsh, project or review is malicious
	Malicious,
	/// Positive neutral, covers rank up to accepted.
	PassedRequirements,
}
/// The status of a proposal sent to the council from here.
#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ProposalStatus {
	pub status: Status,
	pub reason: Reason,
}
/// Default status - storage req
impl Default for Status {
	fn default() -> Self {
		Status::Proposed
	}
}
/// Default reason - storage req
impl Default for Reason {
	fn default() -> Self {
		Reason::PassedRequirements
	}
}
/// The project structure.
#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Project<UserID, Balance> {
	/// The owner of the project
	pub owner_id: UserID,
	/// A bool that allows for simple allocation of the unique chocolate badge. NFT?? (default: false)
	badge: Option<bool>,
	/// Project metadata - req - default some .
	metadata: MetaData,
	/// the status of the project's proposal in the council - default proposed.
	pub proposal_status: ProposalStatus,
	/// A reward value for the project ---------_switch to idea of named reserve hash - (default: Reward).
	pub reward: Balance,
	/// A sum of all the scores of reviews proposed to the project. Saturate when u32::MAX.
	pub total_user_scores: u32,
}
// ------------------------------------------------------------^edit
impl<UserID: Default, Balance: From<u32> + Default> Project<UserID, Balance> {
	///  Set useful defaults.
	///  Initialises a project with defaults on everything except id and metadata
	pub fn new(owner_id: UserID, metadata: MetaData) -> Self {
		Project { owner_id, badge: Option::None, metadata, ..Default::default() }
	}
}
/// A trait that allows project to:
/// - reserve some token for rewarding its reviewers.
pub trait ProjectIO<T: Config> {
	type UserID;
	type Balance;
	/// Checks:
	/// If the projects' reward value reflects what is reserved, excluding existential value
	fn check_reward(project: &Project<Self::UserID, Self::Balance>) -> DispatchResult;
	/// Check if the project owner can offer up hardcoded amount as init.
	fn can_reward(project: &Self::UserID) -> bool;
	/// Reserve an initial amount for use as reward
	fn reserve_reward(project: &mut Project<Self::UserID, Self::Balance>) -> DispatchResult;
	/// Reward the user with an amount and effect edits on the struct level. (Exposes amount in free balance for next step (transfer))
	/// Assumed to be executed right before the final balance transfer
	/// Note: If any failure happens after, reward may be lost.
	fn reward(
		project: &mut Project<Self::UserID, Self::Balance>,
		amount: Self::Balance,
	) -> DispatchResult;
}
/// Easy way of differentaiting the two. We'll need this.
#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
#[cfg_attr(feature = "std", derive(Deserialize, Serialize))]
pub enum EntityKind {
	Project,
	User,
}
