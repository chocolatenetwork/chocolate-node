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
	/// A list of the project's reviewers for validation. (default: 0)
	pub reviewers: Option<Vec<UserID>>,
	/// A list of the project's reviews - Vec (default : none)
	pub reviews: Option<Vec<ReviewID>>,
	/// A bool that allows for simple allocation of the unique chocolate badge. NFT?? (default: false)
	badge: Option<bool>,
	/// Project metadata - req - default some .
	metadata: MetaData,
	/// the status of the project's proposal in the council - default proposed.
	pub proposal_status: ProposalStatus,
	/// A reward value for the project ---------_switch to idea of named reserve hash - (default: Reward).
	pub reward: Balance,
}
// ------------------------------------------------------------^edit
impl<UserID, Balance> Project<UserID, Balance> {
	/// (Trim) Set useful defaults.
	pub fn new(
		owner_id: UserID,
		reviewers: Option<Vec<UserID>>,
		reviews: Option<Vec<ReviewID>>,
		badge: Option<bool>,
		metadata: MetaData,
		proposal_status: ProposalStatus,
		reward: Balance,
	) -> Self {
		Project { owner_id, reviewers, reviews, badge, metadata, proposal_status, reward }
	}
}
/// A trait that allows project to:
/// - reserve some token for rewarding its reviewers.
pub trait ProjectIO<T: Config> {
	type UserID;
	type Balance;

	fn can_reward(project: &Project<Self::UserID, Self::Balance>) -> bool;
	fn reserve_reward(project: &mut Project<Self::UserID, Self::Balance>) -> DispatchResult;
}
