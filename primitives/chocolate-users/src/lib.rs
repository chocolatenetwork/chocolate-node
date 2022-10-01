#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::dispatch::{ DispatchResult};
use frame_system::Config;

#[derive(Encode, Decode, Clone)]
pub struct User {
	pub rank_points: u32,
	pub project_id: Option<u32>,
}
impl Default for User{
	fn default() -> Self{
		// Start from 1 because of total project score calc to avoid accidentally recording zero when we use Default::default()
		User { rank_points: 1, project_id: Option::None }
	}
}
/// UserIO trait for CRUD on users store
pub trait UserIO<T: Config> {
	fn get_user_by_id(id: &T::AccountId) -> Option<User>;
	fn check_owns_project(id: &T::AccountId) -> bool;
	/// Allows us to check if the user even exists before calling get by id.
	fn check_user_exists(id: &T::AccountId) -> bool;
	/// Checks if the user exists, else creates a new user with wanted defaults.
	fn get_or_create_default(id: &T::AccountId) -> User;
	/// Idempotent. Simply creates item in storage if it  doesn't already exist. Use update_user if you'd like to mutate the user after knowing it's been created
	fn set_user(id: &T::AccountId, user: User) -> ();
	fn update_user(id: &T::AccountId, user: User) -> DispatchResult;
}
