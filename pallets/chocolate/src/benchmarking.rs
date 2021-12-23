//! Benchmarking setup for pallet-chocolate

use super::*;

#[allow(unused)]
use crate::Pallet as Chocolate;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
// calls do_something multiple times 0..100 inputs verifying it got stored each time
benchmarks! { // comment for now, do_something is gone, but it's legacy remains
	// do_something {
		// let s in 0 .. 100;
		// let caller: T::AccountId = whitelisted_caller();
	// }: _(RawOrigin::Signed(caller), s)
	// verify {
		// assert_eq!(Something::<T>::get(), Some(s));
	// }
}

impl_benchmark_test_suite!(Chocolate, crate::mock::new_test_ext(), crate::mock::Test);
