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
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_std::mem::{discriminant, Discriminant};
	use sp_std::vec::Vec;
	// Include the ApprovedOrigin type here, and the method to get treasury id, then mint with currencymodule
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		//   Origins that must approve to use the pallet - Should be implemented properly by provider.
		//  In this case pallet_collective implements it as type Origin: From<RawOrigin<Self::AccountId, I>>;
		// type ApprovedOrigin : EnsureOrigin<Self::Origin>;
		// treasury Id??
		// type TreasuryPalletId;
		// The pallet depends on the treasury's definition of proposal id
	}
	/// the treasury's definition of a proposal id - they call it proposal index. u32 as of monthly-08
	/// Deprecated...proposals will reference their actions...indexing through projectID is sufficient
	/// A list of names, an alias for project names
	pub type ListOfNames = Vec<Vec<u8>>;
	/// A simple u32
	pub type ProjectID = u32;
	/// type alias for project socials
	pub type ProjectSocials = Vec<Social>;
	/// Index for reviews , use to link to project
	pub type ReviewID = u64;
	/// type alias for review
	pub type ReviewAl<T> = Review<<T as frame_system::Config>::AccountId>;
	/// type alias for project
	pub type ProjectAl<T> =
		Project<<T as frame_system::Config>::AccountId, <T as frame_system::Config>::Hash>;

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
		/// Custom reason to encapsulate further things like marketCap and other details
		Other(Vec<u8>),
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
		/// A bool that allows for simple allocation of the unique chocolate badge. NFT??
		badge: Option<bool>,
		/// Project metadata
		metadata: MetaData,
		/// the status of the project's proposal in the council.
		proposal_status: ProposalStatus,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Storage map from the project index - id to the projects
	#[pallet::storage]
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
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		/// parameters. [owner,name]
		ProjectCreated(Vec<u8>),
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
	}
}
