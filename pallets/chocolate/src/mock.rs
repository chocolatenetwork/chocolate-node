use crate as pallet_chocolate;
use frame_support::{parameter_types, traits::GenesisBuild};
use frame_system as system;
use pallet_users;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
use chocolate_projects::{Reason, Status};

// The runtime is an enum. omoshiroi
// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		ChocolateModule: pallet_chocolate::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		UsersModule: pallet_users::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::AllowAll;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 500;
}

impl pallet_balances::Config for Test {
	// from treasury tests...not using
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = u128;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}
// ToDo! temp treasury that has implements unbalanced which stores outer state that can be queried

// This is a mock runtime hence we can't avoid importing users and other deps.
/// Configure the pallet-users for UserIO trait
impl pallet_users::Config for Test {
	type Event = Event;
}
parameter_types! {
	pub const Cap: u128 = 5;
	pub const UserCollateral: u128 = 10;
}
// our configs start here
impl pallet_chocolate::Config for Test {
	type Event = Event;
	// no need to rope in collective pallet. we are enough
	type ApprovedOrigin = frame_system::EnsureRoot<u64>;
	// this is simply a pointer to the true implementor,and creator of the currency trait...the balances pallet
	type Currency = Balances;
	type TreasuryOutlet = ();
	type RewardCap = Cap;
	type UsersOutlet = UsersModule;
	type UserCollateral = UserCollateral;
}

// construct a test that mocks treasury runtime but prints imbalance value instead
// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	GenesisConfig {
		//
		balances: BalancesConfig { balances: vec![(1, 5000)] },
		..Default::default()
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
pub fn choc_ext() -> sp_io::TestExternalities {
	let mut t = pallet_chocolate::GenesisConfig::<Test>::default().build_storage().unwrap();

	pallet_chocolate::GenesisConfig::<Test> {
		//
		init_projects: vec![(Status::Accepted, Reason::PassedRequirements)],
		init_users: vec![1,2,3,4,5,6],

		..Default::default()
	}
	.assimilate_storage(&mut t)
	.unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}
