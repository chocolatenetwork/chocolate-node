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

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
		traits::{
			Currency, ExistenceRequirement::KeepAlive, Imbalance, OnUnbalanced, ReservableCurrency,
			WithdrawReasons,
		},
		PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_std::mem::{discriminant, Discriminant};
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
		//  In this case pallet_collective implements it as type Origin: From<RawOrigin<Self::AccountId, I>>;
		// type ApprovedOrigin : EnsureOrigin<Self::Origin>;
		// treasury Id??
		// type TreasuryPalletId;
		// The pallet depends on the treasury's definition of proposal id
	}
	/// type alias for text
	pub type TextAl = Vec<u8>;
	/// A list of names, an alias for project names
	pub type ListOfNames = Vec<Vec<u8>>;
	/// A simple u32
	pub type ProjectID = u32;
	/// type alias for project socials
	pub type ProjectSocials = Vec<Social>;
	/// Index for reviews , use to link to project
	pub type ReviewID = u64;
	/// type alias for review - this is the base struct, like the 2nd part of Balancesof
	pub type ReviewAl<T> = Review<<T as frame_system::Config>::AccountId>;
	/// type alias for project
	pub type ProjectAl<T> =
		Project<<T as frame_system::Config>::AccountId, <T as frame_system::Config>::Hash>;
	/// Type alias for balance, binding T::Currency to Currency::AccountId and then extracting from that Balance. Accessible via T::BalanceOf. T is frame_System.
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	// Due to the complexity of storage, reviews will be limited to n amount. n = 50 . Should be enough to verify a project.
	// runtime types;
	use codec::{Decode, Encode};
	#[derive(Encode, Decode, Default, Clone, PartialEq)]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct Review<UserID> {
		proposal_status: ProposalStatus,
		user_id: UserID,
		review_text: Vec<u8>,
		project_id: ProjectID,
	}
	/// social type, the cfg_Attr is cuz std isn't guaranteed
	/// Socials are equal only if they point to the same string.
	/// This is already implemented by the derive! - PartialEq,
	/// The social enum is complete. I see no reason why vscode is showing err as Vec<u8> is impl by parity
	#[derive(Encode, Decode, Clone, PartialEq)]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub enum Social {
		Twitter(Vec<u8>),
		Facebook(Vec<u8>),
		Instagram(Vec<u8>),
		Riot(Vec<u8>),
		Email(Vec<u8>),
		None,
	}
	/// By default no value
	impl Default for Social {
		fn default() -> Self {
			Social::None
		}
	}
	/// Trait that enforces requirements of projectSocials.
	pub trait ProjectSocialReqs {
		/// Check if a vector contains duplicate instances of an enum variant, regardless of data stored
		fn abstr_dup(&self) -> bool;
		/// Also check if the project has an email
		fn has_email(&self) -> bool;
	}
	impl ProjectSocialReqs for ProjectSocials {
		fn abstr_dup(&self) -> bool {
			// memo for the discriminants
			let mut disc_mem: Vec<Discriminant<Social>> = Vec::new();
			// copy of self for iter
			let cp = (&self).to_vec();
			let mut dupl = false;

			// loop
			for n in cp.iter() {
				// Functions take type arguments as ::<>
				let disc = discriminant::<Social>(n);
				if disc_mem.contains(&disc) {
					dupl = true;
					break;
				};
				disc_mem.push(disc);
			}
			dupl
		}
		fn has_email(&self) -> bool {
			// copy of self for iter
			let cp = (&self).to_vec();
			let mut passed = false;
			let test = Social::Email(b"wasm".to_vec());
			// loop
			for n in cp.iter() {
				// Functions take type arguments as ::<>
				let disc = discriminant::<Social>(n);
				if disc == discriminant::<Social>(&test) {
					passed = true;
					break;
				};
			}
			passed
		}
	}
	/// The metadata of a project. The debug trait is actually a limit of T: Config
	#[derive(Encode, Decode, Default, Clone, PartialEq)]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct MetaData {
		project_name: Vec<u8>,
		/// Vector, preferably a set. In terms of type. Done.
		project_socials: ProjectSocials,
		/// Vector, can contain multiple of same type, just not same value. Allow users to fix such.
		///  It's their responsibility not to try hacking and putting too much. Ui- store as set
		founder_socials: Vec<Social>,
	}

	/// The status of the proposal
	#[derive(Encode, Decode, Clone, PartialEq)]
	#[cfg_attr(feature = "std", derive(Debug))]
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
	pub enum Reason {
		/// Negative lenient - base conditions for project missing or review lacking detail
		InsufficientMetaData,
		/// Negative harsh, project or review is malicious
		Malicious,
		/// Positive neutral, covers rank up to accepted.
		PassedRequirements,
	}
	/// The status of a proposal sent to the council from here. (Unnecessary?)NO. Its call can have a soft limit of any council member.
	#[derive(Encode, Decode, Default, Clone, PartialEq)]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct ProposalStatus {
		/// Doing this to learn pattern matching and stuff. It would also be a good util for reviews.
		status: Status,
		reason: Reason,
	}
	/// Implementing default for the enums, as req by storage
	/// Default status
	impl Default for Status {
		fn default() -> Self {
			Status::Proposed
		}
	}
	/// Default reason
	impl Default for Reason {
		fn default() -> Self {
			Reason::PassedRequirements
		}
	}
	/// The project structure. Initial creation req signed transaction.
	#[derive(Encode, Decode, Default, Clone, PartialEq)]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub struct Project<UserID, Hash> {
		/// The owner of the project
		owner_id: UserID,
		/// A list of the project's reviews - Vec
		reviews: Option<Vec<ReviewID>>,
		/// A hash? that is the badge - ToDo
		badge: Option<Hash>,
		/// Project metadata
		metadata: MetaData,
		/// the status of the project's proposal in the council.
		proposal_status: ProposalStatus,
	}

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
	/// Storage value for project index. Increment as we go
	#[pallet::storage]
	pub type ProjectIndex<T: Config> = StorageValue<_, ProjectID>;
	/// Storage value for reviews index. Increment as we go
	#[pallet::storage]
	pub type ReviewIndex<T: Config> = StorageValue<_, ReviewID>;
	/// Storage value for project names. Keep sorted.
	#[pallet::storage]
	pub type ProjectNames<T: Config> = StorageValue<_, ListOfNames>;
	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId",T::BalanceOf<T> = "Balance")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		/// parameters. [owner,name]
		ProjectCreated(Vec<u8>),
		/// Minted [amount]
		Minted(BalanceOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The project must have at least one email in metadata
		NoEmail,
		/// A project must have at least two means of contact including email
		LessProjectSocials,
		/// Duplicate project socials
		DuplicateProjectSocials,
		/// Insufficient founder socials! Must be >=2
		LessFounderSocials,
		/// The origin dispatched from does not match the owner of the project
		InvalidOwner,
		/// The name given cannot be parsed
		InvalidName,
		/// Another project has the same name
		DuplicateName,
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				}
			}
		}
		// Refactor TO-DO: Abstract validation into a function and generalise.
		/// Create a project
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,3))]
		pub fn create_project(
			origin: OriginFor<T>,
			project_name: TextAl,
			founder_socials: Vec<Social>,
			project_socials: ProjectSocials,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// ensure at least two unique project_Socials and founder_socials, enforce project emails
			ensure!(!(project_socials.abstr_dup()), Error::<T>::DuplicateProjectSocials);
			ensure!(project_socials.has_email(), Error::<T>::NoEmail);
			ensure!(founder_socials.len() >= 2, Error::<T>::LessFounderSocials);
			ensure!(project_socials.len() >= 2, Error::<T>::LessProjectSocials);
			// <Project name validation> - get name for validation
			let name = str::from_utf8(&project_name);
			ensure!(name.is_ok(), Error::<T>::InvalidName);
			// should already be averted...but just in case.
			// Ensure we have an actual value
			let mut name_lower = name.unwrap_or_default().to_lowercase().encode();
			let def: &str = Default::default();
			ensure!(name_lower != def.encode(), Error::<T>::InvalidName);
			// ignore if already lowercase
			if name.unwrap_or_default().to_lowercase() == name.unwrap_or_default() {
				name_lower = name.unwrap_or_default().encode();
			}
			// </Project name validation>

			// ensure no duplicate names.
			let mut names = <ProjectNames<T>>::get().unwrap_or_default();
			match names.binary_search(&name_lower) {
				// because of frame_Dispatch...we use into. Note: outer fn must always return Some(())
				Ok(_) => Err(Error::<T>::DuplicateName.into()),
				Err(index) => {
					// aggregate metadata, and place things in storage
					let met = MetaData { project_name, project_socials, founder_socials };
					let name_lower2 = name_lower.to_vec();
					// Should not panic! since binarysearch should yield appropriate index
					names.insert(index, name_lower);
					<ProjectNames<T>>::put(names);

					let n_index = <ProjectIndex<T>>::get().unwrap_or_default();
					<Projects<T>>::insert(
						n_index.clone(),
						Project {
							owner_id: who,
							reviews: Option::None,
							badge: Option::None,
							metadata: met,
							proposal_status: Default::default(),
						},
					);
					<ProjectIndex<T>>::put(n_index + 1);
					Self::deposit_event(Event::ProjectCreated(name_lower2));
					Ok(())
				}
			}
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
}
