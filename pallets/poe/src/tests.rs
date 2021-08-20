use crate::{mock::*, Error, Proofs};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_claim_works() {
	new_test_ext().execute_with(|| {
		let proof = vec![0, 1];
		let owner = 1;
		assert_ok!(PoeModule::create_claim(Origin::signed(owner), proof.clone()));
		assert_eq!(
			Proofs::<Test>::get(&proof),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		);
	});
}

#[test]
fn create_claim_fails_when_claim_exists() {
	new_test_ext().execute_with(|| {
		let proof = vec![0, 1];
		let owner = 1;
		let _ = PoeModule::create_claim(Origin::signed(owner), proof.clone());

		// re-create claim should throw ProofAlreadyClaimed error
		assert_noop!(
			PoeModule::create_claim(Origin::signed(owner), proof.clone()),
			Error::<Test>::ProofAlreadyClaimed
		);
	});
}

#[test]
fn revoke_claim_works() {
	new_test_ext().execute_with(|| {
		let proof = vec![0, 1];
		let owner = 1;
		let _ = PoeModule::create_claim(Origin::signed(owner), proof.clone());

		assert_ok!(PoeModule::revoke_claim(Origin::signed(owner), proof.clone()));

		// claim shouldn't be found after it's revoked
		assert_eq!(Proofs::<Test>::get(&proof), None);
	});
}

#[test]
fn revoke_claim_fails_when_claim_does_not_exist() {
	new_test_ext().execute_with(|| {
		let proof = vec![0, 1];
		let owner = 1;

		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(owner), proof.clone()),
			Error::<Test>::NoSuchProof
		);
	});
}

#[test]
fn revoke_claim_fails_when_owner_does_not_match() {
	new_test_ext().execute_with(|| {
		let proof = vec![0, 1];
		let owner = 1;
		let other_owner = 2;

		let _ = PoeModule::create_claim(Origin::signed(owner), proof.clone());

		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(other_owner), proof.clone()),
			Error::<Test>::NotProofOwner
		);
	});
}

#[test]
fn transfer_claim_works_with_different_accounts() {
	new_test_ext().execute_with(|| {
		let proof = vec![0, 1];
		let sender = 1;
		let receiver = 2;

		// register the receiver account to system
		let _ = frame_system::Pallet::<Test>::inc_providers(&receiver);

		let _ = PoeModule::create_claim(Origin::signed(sender), proof.clone());

		assert_ok!(PoeModule::transfer_claim(Origin::signed(sender), proof.clone(), receiver));
		assert_eq!(
			Proofs::<Test>::get(&proof),
			Some((receiver, frame_system::Pallet::<Test>::block_number()))
		);
	});
}

#[test]
fn transfer_claim_works_with_self() {
	new_test_ext().execute_with(|| {
		let proof = vec![0, 1];
		let sender = 1;
		let _ = frame_system::Pallet::<Test>::inc_providers(&sender);

		let _ = PoeModule::create_claim(Origin::signed(sender), proof.clone());

		assert_ok!(PoeModule::transfer_claim(Origin::signed(sender), proof.clone(), sender));
		assert_eq!(
			Proofs::<Test>::get(&proof),
			Some((sender, frame_system::Pallet::<Test>::block_number()))
		);
	});
}

#[test]
fn transfer_claim_fails_when_receiver_does_not_exist() {
	new_test_ext().execute_with(|| {
		let proof = vec![0, 1];
		let sender = 1;
		let receiver = 2;

		let _ = PoeModule::create_claim(Origin::signed(sender), proof.clone());

		// do not register receiver to system,
		// which causes the transfer to fail
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(sender), proof.clone(), receiver),
			Error::<Test>::ProofReceiverNotExist
		);
		// the owner of proof should be unchanged
		assert_eq!(
			Proofs::<Test>::get(&proof),
			Some((sender, frame_system::Pallet::<Test>::block_number()))
		);
	});
}

#[test]
fn transfer_claim_fails_when_claim_does_not_exist() {
	new_test_ext().execute_with(|| {
		let proof = vec![0, 1];
		let sender = 1;
		let receiver = 2;
		let _ = frame_system::Pallet::<Test>::inc_providers(&receiver);

		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(sender), proof.clone(), receiver),
			Error::<Test>::NoSuchProof
		);
	});
}

#[test]
fn transfer_claim_fails_when_owner_does_not_match() {
	new_test_ext().execute_with(|| {
		let proof = vec![0, 1];
		let owner = 1;
		let receiver = 2;
		let other_owner = 3;

		let _ = frame_system::Pallet::<Test>::inc_providers(&receiver);
		let _ = PoeModule::create_claim(Origin::signed(owner), proof.clone());

		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(other_owner), proof.clone(), receiver),
			Error::<Test>::NotProofOwner
		);
		// the owner of proof should be unchanged
		assert_eq!(
			Proofs::<Test>::get(&proof),
			Some((owner, frame_system::Pallet::<Test>::block_number()))
		);
	});
}
