use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, assert_err};

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
	});
}

#[test]
fn create_review_should_work() {
	choc_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(ChocolateModule::create_review(Origin::signed(6), [42_u8].to_vec(), 1_u32));
	});
}
#[test]
fn create_review_should_fail() {
	choc_ext().execute_with(|| {
		// Based on current genesis config.
		assert_err!(ChocolateModule::create_review(Origin::signed(1), [40_u8].to_vec(), 1_u32),Error::<Test>::OwnerReviewedProject);
		assert_err!(ChocolateModule::create_review(Origin::signed(2), [40_u8].to_vec(), 1_u32),Error::<Test>::DuplicateReview);
	});
}
