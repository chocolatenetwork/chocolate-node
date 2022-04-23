#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::dispatch::{ DispatchResult};
use frame_system::Config;

#[derive(Encode, Decode, Default, Clone)]
pub struct User {
	pub rank_points: u32,
	pub project_id: Option<u32>,
}

/// UserIO trait for CRUD on users store
pub trait UserIO<T: Config> {
	fn get_user_by_id(id: &T::AccountId) -> Option<User>;
	fn check_owns_project(id: &T::AccountId) -> bool;
	fn check_user_exists(id: &T::AccountId) -> bool;
	/// Checks if the user exists, else creates a new user with wanted defaults.
	fn get_or_create_default(id: &T::AccountId) -> User;
	fn set_user(id: &T::AccountId, user: User) -> ();
	fn update_user(id: &T::AccountId, user: User) -> DispatchResult;
}
