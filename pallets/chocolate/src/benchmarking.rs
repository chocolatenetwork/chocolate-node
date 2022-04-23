//! Benchmarking setup for pallet-chocolate

use super::*;
#[allow(unused)]
use crate::Pallet as Chocolate;
use chocolate_projects::Review;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::traits::{Currency, ExistenceRequirement, Get, Imbalance, OnUnbalanced};
use frame_system::RawOrigin;
// calls do_something multiple times 0..100 inputs verifying it got stored each time
benchmarks! { // comment for now, do_something is gone, but it's legacy remains
	create_review {
		let caller: T::AccountId = whitelisted_caller();
		let s = (b"random").to_vec();
		let prj_id = 1;
		let rev = Review{
			user_id: caller.clone(),
			proposal_status: Default::default(),
			content: s.clone(),
			project_id: prj_id,
			point_snapshot: 12,};
		// costs
		let price = T::Currency::minimum_balance();
		let collat = T::UserCollateral::get();
		let imb =  T::Currency::issue(collat);
		let _ = T::Currency::make_free_balance_be(&caller, imb.peek());
	}: create_review(RawOrigin::Signed(caller), s,prj_id)
	// verify {
	// 	assert_eq!(Reviews::<T>::get(1), Some(rev));
	// }
}

impl_benchmark_test_suite!(Chocolate, crate::mock::new_test_ext(), crate::mock::Test);
