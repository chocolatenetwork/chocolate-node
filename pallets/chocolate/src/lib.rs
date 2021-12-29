#![cfg_attr(not(feature = "std"), no_std)]
/// Study the nicks pallet and modify it after stating its config values to push balances to treasury and have commission control it.

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod constants;

#[frame_support::pallet]
pub mod pallet {
	use crate::constants;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{
			Currency, ExistenceRequirement::KeepAlive, Imbalance, OnUnbalanced, ReservableCurrency,
			WithdrawReasons,
		},
	};
	use frame_system::pallet_prelude::*;
	use sp_std::str;
	use sp_std::vec::Vec;
	// Include the ApprovedOrigin type here, and the method to get treasury id, then mint with currencymodule
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		///  Origins that must approve to use the pallet - Should be implemented properly by provider.
		type ApprovedOrigin: EnsureOrigin<Self::Origin>;
		/// The currency trait, associated to the pallet. All methods accessible from T::Currency*
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// * Treasury outlet: A type with bounds to move slashed funds to the treasury.
		type TreasuryOutlet: OnUnbalanced<NegativeImbalanceOf<Self>>;
		/// * Reward Cap: Max reward projects can place on themselves
		#[pallet::constant]
		type RewardCap: Get<BalanceOf<Self>>;
	}
	pub type NegativeImbalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;
	/// type alias for text
	pub type TextAl = Vec<u8>;
	/// A simple u32
	pub type ProjectID = u32;
	/// Index for reviews , use to link to project
	pub type ReviewID = u64;
	/// type alias for review - this is the base struct, like the 2nd part of Balancesof
	pub type ReviewAl<T> = Review<<T as frame_system::Config>::AccountId>;
	/// type alias for project
	pub type ProjectAl<T> = Project<<T as frame_system::Config>::AccountId,BalanceOf<T>>;
	/// Type alias for balance, binding T::Currency to Currency::AccountId and then extracting from that Balance. Accessible via T::BalanceOf. T is frame_System.
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	use codec::{Decode, Encode};
	#[derive(Encode, Decode, Default, Clone, PartialEq)]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct Review<UserID> {
		proposal_status: ProposalStatus,
		user_id: UserID,
		content: Vec<u8>,
		project_id: ProjectID,
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
		status: Status,
		reason: Reason,
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
	pub struct Project<UserID,Balance> {
		/// The owner of the project
		owner_id: UserID,
		/// A list of the project's reviewers for validation
		reviewers: Option<Vec<UserID>>,
		/// A list of the project's reviews - Vec
		reviews: Option<Vec<ReviewID>>,
		/// A bool that allows for simple allocation of the unique chocolate badge. NFT??
		badge: Option<bool>,
		/// Project metadata
		metadata: MetaData,
		/// the status of the project's proposal in the council.
		proposal_status: ProposalStatus,
		/// A reward value for the project
		reward: Balance,
	}
// ------------------------------------------------------------^edit
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Storage map from the project index - id to the projects. getters are for json rpc.
	#[pallet::storage]
	#[pallet::getter(fn get_projects)]
	pub type Projects<T: Config> = StorageMap<_, Blake2_128Concat, ProjectID, ProjectAl<T>>;
	/// Storage map from the review index - id to the reviews
	#[pallet::storage]
	pub type Reviews<T: Config> = StorageMap<_, Blake2_128Concat, ReviewID, ReviewAl<T>>;
	/// Storage value for project index. Increment as we go.
	/// Analogous to length of project map
	#[pallet::storage]
	pub type ProjectIndex<T: Config> = StorageValue<_, ProjectID>;
	/// Storage value for reviews index. Increment as we go.
	/// Analogous to length of review map
	#[pallet::storage]
	pub type ReviewIndex<T: Config> = StorageValue<_, ReviewID>;
	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		/// parameters. [owner,cid]
		ProjectCreated(T::AccountId, Vec<u8>),
		/// parameters. [owner,project_id]
		ReviewCreated(T::AccountId, ProjectID),
		/// Minted [amount]
		Minted(BalanceOf<T>),
	}
	// ----^edit
	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		/// The project does not exist
		NoProjectWithId,
		/// The reviewer has already placed a review on this project with following id
		DuplicateReview,
		/// The index exceeds max usize.
		StorageOverflow,
		/// Project owners cannot review their projects
		OwnerReviewedProject,
		/// Insufficient funds for rewarding reviewers Do seek a bounty from the treasury. 
		InsufficientBalance,
	}
	// ----------------------------- ^edit
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a project
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,3))]
		pub fn create_project(origin: OriginFor<T>, project_meta: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// CHECKS
			let n_index = <ProjectIndex<T>>::get().unwrap_or_default();
			let new = n_index.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
			// check if project already exists from userIO. if so, Err(you already have one!).
			
			// if balance available, reserve reward, else direct to treasury for bounty.
			let mut project = Project {
					owner_id: who.clone(),
					reviewers: Option::None,
					reviews: Option::None,
					badge: Option::None,
					metadata: project_meta.clone(),
					proposal_status: Default::default(),
					reward: Default::default()
				};
			ProjectIO::<T>::reserve_reward(&mut project)?;	
			// STORAGE MUTATIONS
			<Projects<T>>::insert(
				n_index.clone(),
				project
			);
			<ProjectIndex<T>>::put(new);
			Self::deposit_event(Event::ProjectCreated(who, project_meta));
			Ok(())
		}
		/// Create a review by updating the list of reviewers and reviews of a project and adding review to storage.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,3))]
		pub fn create_review(
			origin: OriginFor<T>,
			review_meta: Vec<u8>,
			project_id: ProjectID,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// CHECKS
			let n_index = <ReviewIndex<T>>::get().unwrap_or_default();
			let new = n_index.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
			let mut this_project =
				<Projects<T>>::get(project_id).ok_or(Error::<T>::NoProjectWithId)?;
			let mut list_of_reviewers = this_project.reviewers.unwrap_or_default();
			let mut list_of_reviews = this_project.reviews.unwrap_or_default();
			ensure!(!list_of_reviewers.contains(&who), Error::<T>::DuplicateReview);
			ensure!(this_project.owner_id.ne(&who), Error::<T>::OwnerReviewedProject);
			// MUTATIONS
			// neither account ids nor the index should exceed max isize
			list_of_reviewers.push(who.clone());
			list_of_reviews.push(n_index.clone());
			// STORAGE MUTATIONS
			this_project.reviewers = Option::Some(list_of_reviewers);
			this_project.reviews = Option::Some(list_of_reviews);
			<Reviews<T>>::insert(
				n_index.clone(),
				Review {
					user_id: who.clone(),
					content: review_meta.clone(),
					project_id: project_id.clone(),
					proposal_status: Default::default(),
				},
			);
			<ReviewIndex<T>>::put(new);
			// update the project
			<Projects<T>>::insert(project_id, this_project);
			Self::deposit_event(Event::ReviewCreated(who, project_id));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn mint(origin: OriginFor<T>, x: BalanceOf<T>) -> DispatchResult {
			// call its ensure origin
			let _who = T::ApprovedOrigin::ensure_origin(origin)?;
			// then check subsume our balance - ToDo
			
			Self::deposit_event(Event::Minted(x.clone()));
			Ok(())
		}

	}
	/// A trait that allows project to:
	/// - reserve some token for rewarding its reviewers.
	pub trait ProjectIO<T:Config>{
		fn can_reward(&self)->bool;
		fn reserve_reward(&mut self) -> DispatchResult;
	}
	impl<T:Config> ProjectIO<T> for ProjectAl<T>{
		fn can_reward(&self)->bool{
			T::Currency::can_reserve(&self.owner_id, T::RewardCap::get())
		}
		fn reserve_reward(&mut self) -> DispatchResult{
			ensure!(ProjectIO::<T>::can_reward(self),Error::<T>::InsufficientBalance);
			T::Currency::reserve(&self.owner_id, T::RewardCap::get())?;
			self.reward = T::RewardCap::get();
			Ok(())
		}
	}
	
	/// A separate impl pallet<T> for custom functions that aren't extrinsics
	impl<T: Config> Pallet<T> {
		
		/// Create a project from required data - only for genesis
		pub fn initialize_projects(
			this_owner_id: T::AccountId,
			this_meta: Vec<u8>,
			this_revs: Vec<ReviewID>,
			this_reviewers: Vec<T::AccountId>,
			this_status: Status,
			this_reason: Reason,
		) -> ProjectAl<T> {
			let mut returnable = Project {
				owner_id: this_owner_id.clone(),
				reviewers: Option::Some(this_reviewers),
				reviews: Option::Some(this_revs),
				badge: Option::None,
				metadata: this_meta,
				proposal_status: ProposalStatus { status: this_status, reason: this_reason },
				reward: Default::default(),
			};
			let res= ProjectIO::<T>::reserve_reward(&mut returnable);
			if !res.is_ok(){
				// temporary hack to ensure we have enough. Figure out a way of directly issuing from the treasury without spend some funds and co. for this genesis. And give the treasury some funds!
                let imbalance =T::Currency::issue( T::RewardCap::get());
                let imbalance2 =T::Currency::issue( T::RewardCap::get());

				T::Currency::resolve_creating(&this_owner_id, imbalance);
				T::Currency::resolve_creating(&this_owner_id, imbalance2);
				let _= ProjectIO::<T>::reserve_reward(&mut returnable);
			}

			returnable
		}
		pub fn initialize_reviews(acnt_ids: Vec<T::AccountId>) -> Vec<ReviewID> {
			let clns = acnt_ids.iter().clone();
			let mut n_index = <ReviewIndex<T>>::get().unwrap_or_default();
			let n_proj = <ProjectIndex<T>>::get().unwrap_or_default();
			// intialize review contents with their ids
			let list_of_revs: Vec<ReviewAl<T>> = constants::project::REVS
				.iter()
				.clone()
				.zip(clns)
				.map(|(rev, id)| Review {
					project_id: n_proj,
					proposal_status: ProposalStatus {
						status: Status::Accepted,
						reason: Default::default(),
					},
					content: rev.to_vec(),
					user_id: id.clone(),
				})
				.collect();
			// storage mutations
			let mut list_of_indexes: Vec<ReviewID> = Vec::new();
			for elem in list_of_revs.iter() {
				// shouldn't panic because we aren't placing more than four in.
				<Reviews<T>>::insert(n_index, elem);
				list_of_indexes.push(n_index.clone());
				n_index += 1;
			}
			<ReviewIndex<T>>::put(n_index);
			return list_of_indexes;
		}
	}
	/// Genesis config for the chocolate pallet
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		/// Get the parameters for the init projects function
		pub init_projects: Vec<(T::AccountId, Status, Reason)>,
	}
	/// By default a generic project or known projects will be shown - polkadot & sisters
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			// to-do actually make this known projects. In the meantime, default will do.
			Self { init_projects: Vec::new() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// setup a counter to serve as project index
			let mut count: ProjectID = 0;
			let meta: Vec<Vec<u8>> =
				constants::project::METADATA.iter().map(|each| each.to_vec()).collect();
			let zipped = (&self.init_projects).into_iter().clone().zip(meta.iter().clone());
			// create project from associated metadata in zip.
			for each in zipped {
				let (this_project, meta_ref) = each.to_owned();
				let meta_cid = meta_ref.to_owned();
				let (acnt, stat, reas) = this_project.to_owned();
				// Filter ids so generated reviews do not include project owner
				let filtered_ids: Vec<T::AccountId> = (&self.init_projects)
					.into_iter()
					.clone()
					.filter(|(id, ..)| acnt.ne(id))
					.map(|long| long.0.clone())
					.collect();
				// create reviews and projects and store.
				let review_ids: Vec<ReviewID> =
					Pallet::<T>::initialize_reviews(filtered_ids.clone());
				let returnable = Pallet::<T>::initialize_projects(
					acnt,
					meta_cid,
					review_ids,
					filtered_ids,
					stat,
					reas,
				);
				<Projects<T>>::insert(count.clone(), returnable);
				count += 1;
				<ProjectIndex<T>>::put(count);
			}
		}
	}
}
