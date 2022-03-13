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
	use chocolate_projects::*;
	use chocolate_users::UserIO;
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		sp_runtime::traits::Saturating,
		traits::{
			Currency, ExistenceRequirement::KeepAlive, Imbalance, OnUnbalanced, ReservableCurrency,
		},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{traits::CheckedDiv, ArithmeticError};
	use sp_std::{str, vec::Vec};
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
		/// This is it! The user pallet. A type with bounds to access the user module.
		type UsersOutlet: UserIO<Self>;
		/// * Reward Cap: Max reward projects can place on themselves. Interestingly, this also serves as their stake amount.
		#[pallet::constant]
		type RewardCap: Get<BalanceOf<Self>>;
		/// Hard coded collateral amount for the Users
		#[pallet::constant]
		type UserCollateral: Get<BalanceOf<Self>>;
	}
	// ------------------------------------------------------------Type aliases ---------------------\
	pub type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::NegativeImbalance;
	/// type alias for review - this is the base struct, like the 2nd part of Balancesof
	pub type ReviewAl<T> = Review<<T as frame_system::Config>::AccountId>;
	/// type alias for project
	pub type ProjectAl<T> = Project<<T as frame_system::Config>::AccountId, BalanceOf<T>>;
	/// Type alias for balance, binding T::Currency to Currency::AccountId and then extracting from that Balance. Accessible via T::BalanceOf. T is frame_System.
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Storage map from the project index - id to the projects. getters are for json rpc.
	#[pallet::storage]
	#[pallet::getter(fn get_projects)]
	pub type Projects<T: Config> = StorageMap<_, Blake2_128Concat, ProjectID, ProjectAl<T>>;
	/// Storage double map from the userid and projectid to the reviews.
	/// I.e A user owns many reviews,each belonging to a unique project.
	#[pallet::storage]
	pub type Reviews<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		ProjectID,
		ReviewAl<T>,
	>;
	/// Storage value for project index. Increment as we go.
	/// Analogous to 1+length of project map. it starts at 1.
	#[pallet::storage]
	pub type NextProjectIndex<T: Config> = StorageValue<_, ProjectID>;
	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		/// parameters. [owner,cid,project_id]
		ProjectCreated(T::AccountId, Vec<u8>, ProjectID),
		/// parameters. [owner,project_id]
		ReviewCreated(T::AccountId, ProjectID),
		/// parameters [owner,id]
		ReviewAccepted(T::AccountId, ProjectID),
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
		/// Insufficient funds for performing a task. Add more funds to your account/call/reserve.
		InsufficientBalance,
		/// The reward on the project isn't same as reserve
		RewardInconsistent,
		/// User already owns a project
		AlreadyOwnsProject,
		/// The collateral for the review is not present
		InconsistentCollateral,
		/// The review matching this key cannot be found
		ReviewNotFound,
		/// The call to accept must be on a proposed review with appropriate state
		AcceptingNotProposed,
		/// The checked division method failed, either due to overflow/underflow or because of division by zero.
		CheckedDivisionFailed,
	}
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a project
		///  
		/// - O(1).  
		/// - Init: Index starts at 0
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,3))]
		pub fn create_project(origin: OriginFor<T>, project_meta: Vec<u8>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// CHECKS
			let index = <NextProjectIndex<T>>::get().unwrap_or(1);
			let new_index = index.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
			let mut user = T::UsersOutlet::get_or_create_default(&who).unwrap_or_default();
			let user_project_id = user.project_id.unwrap_or_default();
			// default for u32 is 0 but projectID starts at 1
			ensure!(user_project_id == 0, Error::<T>::AlreadyOwnsProject);
			ensure!(Pallet::<T>::can_reward(&who), Error::<T>::InsufficientBalance);
			// Init structs.
			let mut project = ProjectAl::<T>::new(who.clone(), project_meta.clone());
			// FALLIBLE MUTATIONS
			Pallet::<T>::reserve_reward(&mut project)?;
			user.project_id = Some(new_index);
			// STORAGE MUTATIONS
			<Projects<T>>::insert(index, project);
			<NextProjectIndex<T>>::put(new_index);
			T::UsersOutlet::update_user(&who, user).expect("User should already exist");
			Self::deposit_event(Event::ProjectCreated(who, project_meta, index));
			Ok(())
		}
		/// Create a review, reserve required collateral and increase total of user trust scores on project.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,3))]
		pub fn create_review(
			origin: OriginFor<T>,
			review_meta: Vec<u8>,
			project_id: ProjectID,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// CHECKS & Inits
			let mut this_project =
				<Projects<T>>::get(project_id).ok_or(Error::<T>::NoProjectWithId)?;
			ensure!(!<Reviews<T>>::contains_key(&who, project_id), Error::<T>::DuplicateReview);
			ensure!(this_project.owner_id.ne(&who), Error::<T>::OwnerReviewedProject);
			let reserve = Pallet::<T>::can_collateralise(&who)?;
			// Fallible MUTATIONS
			Pallet::<T>::collateralise(&who, reserve)?;
			let user = T::UsersOutlet::get_or_create_default(&who)?;
			this_project.total_user_scores =
				this_project.total_user_scores.saturating_add(user.rank_points);
			// STORAGE MUTATIONS
			<Reviews<T>>::insert(
				who.clone(),
				project_id,
				Review {
					user_id: who.clone(),
					content: review_meta,
					project_id,
					proposal_status: Default::default(),
					point_snapshot: user.rank_points,
				},
			);
			<Projects<T>>::mutate(project_id, |project| {
				*project = Some(this_project);
			});
			Self::deposit_event(Event::ReviewCreated(who, project_id));
			Ok(())
		}
		/// Releases collateral and rewards user for a good review.
		///
		/// **Call requirements**:
		/// - Origin must be cacao
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,3))]
		pub fn accept_review(
			origin: OriginFor<T>,
			user_id: T::AccountId,
			project_id: ProjectID,
		) -> DispatchResult {
			T::ApprovedOrigin::ensure_origin(origin)?;
			// Values
			let mut review =
				<Reviews<T>>::get(&user_id, project_id).ok_or(Error::<T>::ReviewNotFound)?;
			let mut project = <Projects<T>>::get(project_id).ok_or(Error::<T>::NoProjectWithId)?;
			// CHECKS
			let is_proposed = review.proposal_status.status.eq(&Status::Proposed);
			ensure!(is_proposed, Error::<T>::AcceptingNotProposed);
			ensure!(Pallet::<T>::check_collateral(&user_id), Error::<T>::InconsistentCollateral);
			Pallet::<T>::check_reward(&project)?;
			// MUTATIONS - Fallible
			Pallet::<T>::reward_user(&user_id, &mut project, &review)?;
			review.proposal_status.status = Status::Accepted;
			review.proposal_status.reason = Reason::PassedRequirements;
			// STORAGE MUTATIONS
			<Reviews<T>>::mutate(&user_id, project_id, |r| {
				*r = Option::Some(review);
			});
			<Projects<T>>::mutate(project_id, |p| {
				*p = Option::Some(project);
			});
			Self::deposit_event(Event::ReviewCreated(user_id, project_id));
			Ok(())
		}
		#[pallet::weight(10_000)]
		pub fn mint(origin: OriginFor<T>, x: BalanceOf<T>) -> DispatchResult {
			// call its ensure origin - doesn't return origin. Only checks
			T::ApprovedOrigin::ensure_origin(origin)?;
			let imbalance = T::Currency::issue(x);
			let minted = imbalance.peek();
			Self::do_mint(imbalance);
			Self::deposit_event(Event::Minted(minted));
			Ok(())
		}
	}

	impl<T: Config> ProjectIO<T> for Pallet<T> {
		type UserID = T::AccountId;
		type Balance = BalanceOf<T>;
		/// does existence checks for us to see if the project owner can release required reward+liveness req safely
		fn can_reward(who: &Self::UserID) -> bool {
			let existential = T::Currency::minimum_balance();
			let mut amount = T::RewardCap::get();
			let reserved = T::Currency::reserved_balance(who);
			// modulo test two
			if (reserved % amount) < existential {
				amount = amount.saturating_add(existential);
			}
			T::Currency::can_reserve(who, amount)
		}
		/// Performs the necessary checks on the project's side to ensure that they can reward the user
		/// At this instance
		///
		/// - checks if the project's advertised reward is same as reserved balance
		/// - checks if the project has enough free balance to safely transfer reward after release to the user
		fn check_reward(project_struct: &ProjectAl<T>) -> DispatchResult {
			// If the reward is what is reserved + existential
			let reserve = T::Currency::reserved_balance(&project_struct.owner_id); // If we had the reward, taking the modulo with reward cap would yield what we have??
			let existential = T::Currency::minimum_balance();
			// assume reserve to be a superset of our reward.
			let is_sufficient = reserve >= (project_struct.reward.saturating_add(existential));
			ensure!(is_sufficient, Error::<T>::RewardInconsistent);
			// ensure free balance too for the next step
			let free_balance = T::Currency::free_balance(&project_struct.owner_id);
			ensure!(free_balance >= existential, Error::<T>::InsufficientBalance);
			Ok(())
		}
		/// Reserve some reward and possibly + liveness req as we initialise the project
		/// # Fallible
		/// does no checks for ability to reserve.
		/// (When safe, move from mut to immut)
		fn reserve_reward(project_struct: &mut ProjectAl<T>) -> DispatchResult {
			let existential = T::Currency::minimum_balance();
			let mut amount = T::RewardCap::get();
			let reserved = T::Currency::reserved_balance(&project_struct.owner_id);
			// modulo test two
			if (reserved % amount) < existential {
				amount = amount.saturating_add(existential);
			}
			T::Currency::reserve(&project_struct.owner_id, amount)?;
			project_struct.reward = T::RewardCap::get();
			Ok(())
		}
		/// * Releases an amount from the reward reserved with the project
		/// Assumed to be called in the context of rewarding the user wherein amount
		/// is the final reward calculated.
		/// * **Does no checks**. Assumes specific state of review and project . i.e default Proposed and Accepted states respectively.
		/// * Since unreserve doesn't have an expect block, if there is remaining balance, we assume error and rollback
		/// 	 # Panics!
		///  with expect if we can't rollback, and returns dispatch error for inconsistent reward.
		/// **Requires** : check_reward , check_collateral, (caller) reward_user (In context of accept reward)
		fn reward(project_struct: &mut ProjectAl<T>, amount: Self::Balance) -> DispatchResult {
			// MUTATIONS
			let _missing_reward = T::Currency::unreserve(&project_struct.owner_id, amount);
			if _missing_reward > BalanceOf::<T>::from(0u32) {
				// assuming our can_unreserve failed
				// rollback ----
				T::Currency::reserve(
					&project_struct.owner_id,
					amount.saturating_sub(_missing_reward),
				)
				.expect("Should be enough to rollback following our initial unreserve");
				return Err(Error::<T>::RewardInconsistent.into())
			}
			// Update the reward on project.
			project_struct.reward = project_struct.reward.saturating_sub(amount);
			Ok(())
		}
	}

	/// A separate impl pallet<T> for custom functions that aren't extrinsics
	impl<T: Config> Pallet<T> {
		/// checks if the user's collateral is complete and sufficient for the rewarding process.
		/// Assumed to be used in context where we'll be using this collateral balance immediately.
		/// E.g for rewarding
		pub fn check_collateral(who: &T::AccountId) -> bool {
			let collateral = T::UserCollateral::get();
			let existential_deposit = T::Currency::minimum_balance();
			let reserve = T::Currency::reserved_balance(who);
			reserve >= (collateral.saturating_add(existential_deposit))
		}
		/// Release the collateral held by the account. Should only be called in the context of acceptance.
		/// Does no checks. Assumes the state is as required.
		///
		/// **Requires** : check_collateral
		pub fn release_collateral(who: &T::AccountId) -> DispatchResult {
			T::Currency::unreserve(&who, T::UserCollateral::get());
			Ok(())
		}
		/// Reward the user for their contribution to the project. Assumed to be called after acceptance.
		///
		/// **requires**: check_reward and check_collateral
		pub fn reward_user(
			who: &T::AccountId,
			project: &mut ProjectAl<T>,
			review: &ReviewAl<T>,
		) -> DispatchResult {
			let reward = project.reward.clone();
			// Reward calc
			// reward is reward * (user_point/ttl_project_point )-- use fixed point attr of BalanceOf and move vars around in eqn.

			let balance_prj_score = BalanceOf::<T>::from(project.total_user_scores);
			let balance_rev_sshot = BalanceOf::<T>::from(review.point_snapshot);
			let balance_div = reward
				.checked_div(&balance_prj_score)
				.ok_or(DispatchError::Arithmetic(ArithmeticError::DivisionByZero))?;

			let reward_fraction = balance_div.saturating_mul(balance_rev_sshot);
			// Unreserve our final decision from project.
			// We expect projects to not edit this reserve. What if they do?? - Users tx start failing: Ask users to Report! if found, and track txs

			// Mutations
			Pallet::<T>::release_collateral(who)?;
			Pallet::<T>::reward(project, reward_fraction).expect("should be able to reward"); // nothing should fail after release
			T::Currency::transfer(&project.owner_id, who, reward_fraction, KeepAlive)
				.expect("should be enough to safely transfer");
			Ok(())
		}
		/// Check if a **user** can serve up the required collateral
		///
		/// includes existential requirement for reserved balance if it doesn't already exist.
		///
		/// Returns the amount of collateral after performing checks
		pub fn can_collateralise(id: &T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
			let mut reserve = T::UserCollateral::get();
			// check if existential deposit already exists in reserve, add to balance to reserve if not
			let existential_deposit = T::Currency::minimum_balance();
			let reserved = T::Currency::reserved_balance(id);
			if (reserved % reserve) < existential_deposit {
				reserve = reserve.saturating_add(existential_deposit);
			}
			let can_reserve = T::Currency::can_reserve(id, reserve);
			if can_reserve {
				Ok(reserve)
			} else {
				Err(Error::<T>::InsufficientBalance.into())
			}
		}
		/// Reserve a specific amount for the review.
		///
		/// Assumes checks have already been made for the specified amount.
		/// Requires `can_collateralise`
		pub fn collateralise(id: &T::AccountId, reserve: BalanceOf<T>) -> DispatchResult {
			T::Currency::reserve(&id, reserve)?;
			Ok(())
		}
		/// Function to take negative imbalance to the treasury, expected to be called after creating one e.g through T::Currency::issue()
		pub fn do_mint(amount: NegativeImbalanceOf<T>) {
			T::TreasuryOutlet::on_unbalanced(amount);
		}

		/// Create a project from required data - only for genesis
		/// Assumes user has already been craeted.
		/// # Panics
		/// Panics with expect block if it cannot update the user or reserve the reward amount.
		pub fn initialize_project(
			who: T::AccountId,
			metadata: Vec<u8>,
			status: Status,
			reason: Reason,
			count: ProjectID,
		) -> ProjectAl<T> {
			let mut project = ProjectAl::<T>::new(who.clone(), metadata);
			let mut user = T::UsersOutlet::get_user_by_id(&who).unwrap_or_default();
			// FALLIBLE MUTATIONS
			Pallet::<T>::reserve_reward(&mut project)
				.expect("The project owner should have sufficient balance");
			user.project_id = Some(count);
			project.proposal_status.status = status;
			project.proposal_status.reason = reason;
			// STORAGE MUTATIONS
			<Projects<T>>::insert(count, project.clone());
			// <NextProjectIndex<T>>::put(count.saturating_add(1));
			T::UsersOutlet::update_user(&who, user).expect("User should exist");
			project
		}
		/// Create a set of reviews from a set of ids as needed and places them in storage
		pub fn initialize_reviews(
			acnt_ids: Vec<T::AccountId>,
			project: &mut ProjectAl<T>,
			count: ProjectID,
		) -> Vec<ReviewAl<T>> {
			let acnt_ids_iter = acnt_ids.iter();
			let mut local_pt = count;
			// 15 is our target "gp". Pseudo random. This seed seems good enough.
			let mut spread_points = || {
				local_pt = local_pt.saturating_add(local_pt.saturating_add(7));
				local_pt = local_pt.saturating_mul(17) % 15u32;
				if local_pt == 0 {
					local_pt = local_pt.saturating_add(7);
				}
				local_pt
			};
			// intialize review contents with their ids
			let list_of_revs: Vec<ReviewAl<T>> = constants::project::REVS
				.iter()
				.zip(acnt_ids_iter)
				.map(|(rev, id)| {
					let reserve = Pallet::<T>::can_collateralise(id).expect(
						"The user should have the required balance, enough to avoid reaping too",
					);
					let _ = Pallet::<T>::collateralise(id, reserve);
					// force collateralise each so we can immediately apply accept i.e update stake on project and supply reward.
					let mut user = T::UsersOutlet::get_user_by_id(id).unwrap_or_default();

					user.rank_points = spread_points();
					// init rev
					let mut review: ReviewAl<T> = Default::default();
					review.project_id = count;
					review.proposal_status.status = Status::Accepted;
					review.content = rev.to_vec();
					review.user_id = id.clone();
					review.point_snapshot = user.rank_points;
					project.total_user_scores =
						project.total_user_scores.saturating_add(user.rank_points);

					T::UsersOutlet::update_user(id, user).expect("User should exist");
					<Reviews<T>>::insert(id.clone(), count, review.clone());

					review
				})
				.collect();

			// storage mutations
			<Projects<T>>::mutate(count, |p| *p = Some(project.clone()));
			for elem in list_of_revs.iter() {
				let _ = Pallet::<T>::reward_user(&elem.user_id, project, &elem)
					.expect("The collateral and all exists");
				<Projects<T>>::mutate(count, |p| *p = Some(project.clone()));
			}
			list_of_revs
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
			let mut count: ProjectID = 1;
			let meta: Vec<Vec<u8>> =
				constants::project::METADATA.iter().map(|each| each.to_vec()).collect();
			let zipped = (&self.init_projects).into_iter().zip(meta.iter());
			// create project from associated metadata in zip.
			for each in zipped {
				let (this_project, meta_ref) = each;
				let meta_cid = meta_ref.to_owned();
				let (acnt, stat, reas) = this_project.to_owned();
				// Filter ids so generated reviews do not include project owner
				let filtered_ids: Vec<_> = (&self.init_projects)
					.into_iter()
					.filter(|(id, ..)| acnt.ne(id))
					.map(|long| long.0.clone())
					.collect();
				// Give filtered ids and main acnt enough funds to pay for reward.
				//  (Hack). More formal ways should be decided upon.

				let two = BalanceOf::<T>::from(2u32);
				let minimum = T::Currency::minimum_balance();
				let amnt_issue = T::RewardCap::get().saturating_mul(two).saturating_add(minimum);
				let amnt_issue2 =
					T::UserCollateral::get().saturating_mul(two).saturating_add(minimum);
				let total = amnt_issue.saturating_add(amnt_issue2).saturating_mul(two);
				let imbalance = T::Currency::issue(total.clone());
				T::Currency::resolve_creating(&acnt, imbalance);
				filtered_ids
					.iter()
					.for_each(|id| T::Currency::resolve_creating(id, T::Currency::issue(total)));
				// create the users.
				filtered_ids.iter().for_each(|id| {
					T::UsersOutlet::get_or_create_default(&acnt).expect("Should complete");
					T::UsersOutlet::get_or_create_default(id).expect("Should complete");
				});
				// create reviews and projects and store.
				let mut returnable =
					Pallet::<T>::initialize_project(acnt, meta_cid, stat, reas, count);
				let _reviews: Vec<_> =
					Pallet::<T>::initialize_reviews(filtered_ids, &mut returnable, count);
				// STORAGE MUTATIONS -- after due to mut
				count += 1;
				<NextProjectIndex<T>>::put(count);
			}
			// Fill the treasury - A little hack.
			let imbalance = T::Currency::issue(T::RewardCap::get());
			Pallet::<T>::do_mint(imbalance);
		}
	}
}
