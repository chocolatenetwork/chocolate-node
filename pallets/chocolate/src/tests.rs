use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(ChocolateModule::do_something(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(ChocolateModule::something(), Some(42));
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(ChocolateModule::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
	});
}

#[test]
fn create_project_should_work() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(ChocolateModule::create_project(Origin::signed(1), [42_u8].to_vec()));
	});
}

#[test]
fn create_project_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(ChocolateModule::create_project(Origin::signed(1), [40_u8].to_vec()));
	})
}

// #[test]
// fn create_review_should_work() {
// 	new_test_ext().execute_with(|| {
// 		// Dispatch a signed extrinsic.
// 		assert_ok!(ChocolateModule::create_review(Origin::signed(1), [42_u8].to_vec(), 42_u32));
// 	});
// }

// fn create_review_should_fail() {
// 	new_test_ext().execute_with(|| {
// 		// Dispatch a signed extrinsic.
// 		assert_ok!(ChocolateModule::create_review(Origin::signed(1), [40_u8].to_vec(), 42_u32));
// 	});
// }