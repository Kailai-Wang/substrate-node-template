use crate::{mock::*, Error};
use frame_support::{assert_err_ignore_postinfo, assert_noop, assert_ok, traits::Currency};

// block numbers are already set in new_test_ext()

#[test]
fn create_kitty_works() {
	new_test_ext().execute_with(|| {
		let _ = <Balances as Currency<_>>::deposit_creating(&1, 100);
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_eq!(KittiesModule::current_kitty_index(), 1);

		// check the kitty status
		let kitty = KittiesModule::kitties(1);
		assert!(kitty.is_some());
		let kitty = kitty.unwrap();
		assert_eq!(kitty.index, 1);
		assert_eq!(kitty.price, 0);
		assert_eq!(kitty.creator, 1);

		// check the kitty owner
		let owner = KittiesModule::owner(1);
		assert!(owner.is_some());
		assert_eq!(owner.unwrap(), 1);

		// check balance of the creator
		// we should have only 90 as "free" balance, as 10 was reserved when creating kitties
		assert_eq!(<Balances as Currency<_>>::free_balance(&1), 90);

		// check event
		System::assert_last_event(Event::KittiesModule(crate::Event::KittyCreated(1, 1)));
	});
}

#[test]
fn create_kitty_fails_with_no_initial_balance() {
	new_test_ext().execute_with(|| {
		let result = KittiesModule::create(Origin::signed(1));
		assert_err_ignore_postinfo!(result, pallet_balances::Error::<Test>::InsufficientBalance);
	});
}

#[test]
fn create_kitty_fails_with_index_overflow() {
	new_test_ext().execute_with(|| {
		let _ = <Balances as Currency<_>>::deposit_creating(&1, 100);
		KittiesModule::set_kitty_index(KittyIndex::max_value());
		assert_noop!(KittiesModule::create(Origin::signed(1)), Error::<Test>::KittyIndexOverflow);
	});
}

#[test]
fn transfer_kitty_works() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let receiver = 2;
		let _ = <Balances as Currency<_>>::deposit_creating(&sender, 100);
		let _ = KittiesModule::create(Origin::signed(sender));

		assert_ok!(KittiesModule::transfer(Origin::signed(sender), receiver, 1));

		// kitty creator should not be changed
		let kitty = KittiesModule::kitties(1);
		assert!(kitty.is_some());
		assert_eq!(kitty.unwrap().creator, sender);

		// check the kitty owner again
		let owner = KittiesModule::owner(1);
		assert!(owner.is_some());
		assert_eq!(owner.unwrap(), receiver);

		// check event
		System::assert_last_event(Event::KittiesModule(crate::Event::KittyTransferred(
			sender, receiver, 1,
		)));
	});
}

#[test]
fn transfer_kitty_fails_with_wrong_owner() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let receiver = 2;
		let _ = <Balances as Currency<_>>::deposit_creating(&sender, 100);
		let _ = KittiesModule::create(Origin::signed(sender));
		let non_exist_index = 3;
		assert_noop!(
			KittiesModule::transfer(Origin::signed(sender), receiver, non_exist_index),
			Error::<Test>::NotKittyOwner
		);
	});
}

#[test]
fn breed_kitty_works() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let _ = <Balances as Currency<_>>::deposit_creating(&sender, 100);
		// create two kitties
		let _ = KittiesModule::create(Origin::signed(sender));
		let _ = KittiesModule::create(Origin::signed(sender));

		assert_ok!(KittiesModule::breed(Origin::signed(sender), 1, 2));
		// now should have 3 kitties
		assert_eq!(KittiesModule::current_kitty_index(), 3);

		// double check the owner
		let owner = KittiesModule::owner(3);
		assert!(owner.is_some());
		assert_eq!(owner.unwrap(), sender);

		// check event
		System::assert_last_event(Event::KittiesModule(crate::Event::KittyBred(sender, 3)));
	});
}

#[test]
fn breed_kitty_fails_with_same_parent_index() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let _ = <Balances as Currency<_>>::deposit_creating(&sender, 100);
		// create two kitties
		let _ = KittiesModule::create(Origin::signed(sender));
		let _ = KittiesModule::create(Origin::signed(sender));

		assert_noop!(
			KittiesModule::breed(Origin::signed(sender), 2, 2),
			Error::<Test>::SameParentIndex
		);
	});
}

#[test]
fn breed_kitty_fails_with_no_such_kitty_index() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let _ = <Balances as Currency<_>>::deposit_creating(&sender, 100);
		// create two kitties
		let _ = KittiesModule::create(Origin::signed(sender));
		let _ = KittiesModule::create(Origin::signed(sender));

		assert_noop!(
			KittiesModule::breed(Origin::signed(sender), 2, 3),
			Error::<Test>::NoSuchKittyIndex
		);
	});
}

#[test]
fn sell_kitty_works() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let _ = <Balances as Currency<_>>::deposit_creating(&sender, 100);
		let _ = KittiesModule::create(Origin::signed(sender));

		assert_ok!(KittiesModule::sell(Origin::signed(sender), 1, 10));

		// check the kitty status
		let kitty = KittiesModule::kitties(1);
		assert!(kitty.is_some());
		let kitty = kitty.unwrap();
		assert_eq!(kitty.index, 1);
		assert_eq!(kitty.price, 10);
		assert!(kitty.is_for_sale);

		// check event
		System::assert_last_event(Event::KittiesModule(crate::Event::KittyOnSale(1, 10)));
	});
}

#[test]
fn sell_kitty_fails_with_invalid_sell_price() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let _ = <Balances as Currency<_>>::deposit_creating(&sender, 100);
		let _ = KittiesModule::create(Origin::signed(sender));

		assert_noop!(
			KittiesModule::sell(Origin::signed(sender), 1, 0), // set sell price to 0
			Error::<Test>::InvalidSellPrice
		);
	});
}

#[test]
fn sell_kitty_fails_with_wrong_owner() {
	new_test_ext().execute_with(|| {
		let sender = 1;
		let _ = <Balances as Currency<_>>::deposit_creating(&sender, 100);
		let _ = KittiesModule::create(Origin::signed(sender));

		assert_noop!(
			KittiesModule::sell(Origin::signed(sender), 2, 10),
			Error::<Test>::NotKittyOwner
		);
	});
}

#[test]
fn buy_kitty_works() {
	new_test_ext().execute_with(|| {
		let seller = 1;
		let buyer = 2;

		let _ = <Balances as Currency<_>>::deposit_creating(&seller, 100);
		let _ = <Balances as Currency<_>>::deposit_creating(&buyer, 100);

		let _ = KittiesModule::create(Origin::signed(seller));
		assert_eq!(<Balances as Currency<_>>::free_balance(&seller), 90);

		// put the kitty on sale
		let _ = KittiesModule::sell(Origin::signed(seller), 1, 10);

		// buy it
		assert_ok!(KittiesModule::buy(Origin::signed(buyer), 1));

		// check the kitty status
		let kitty = KittiesModule::kitties(1);
		assert!(kitty.is_some());
		let kitty = kitty.unwrap();
		assert_eq!(kitty.index, 1);
		assert_eq!(kitty.price, 10);
		assert!(!kitty.is_for_sale); // after it's bought it should be false

		// check the kitty owner
		let owner = KittiesModule::owner(1);
		assert!(owner.is_some());
		assert_eq!(owner.unwrap(), buyer);

		// check deposit
		assert_eq!(<Balances as Currency<_>>::free_balance(&seller), 110); // 90 + 10 + 10
		assert_eq!(<Balances as Currency<_>>::free_balance(&buyer), 90);

		// check event
		System::assert_last_event(Event::KittiesModule(crate::Event::KittyBought(1, buyer)));
	});
}

#[test]
fn buy_kitty_fails_if_kitty_is_not_for_sale() {
	new_test_ext().execute_with(|| {
		let seller = 1;
		let buyer = 2;

		let _ = <Balances as Currency<_>>::deposit_creating(&seller, 100);
		let _ = <Balances as Currency<_>>::deposit_creating(&buyer, 100);

		let _ = KittiesModule::create(Origin::signed(seller));
		assert_eq!(<Balances as Currency<_>>::free_balance(&seller), 90);

		// buy it without seller first putting it for sale
		assert_noop!(KittiesModule::buy(Origin::signed(buyer), 1), Error::<Test>::KittyNotForSale);
	});
}

#[test]
fn buy_kitty_fails_if_kitty_not_exist() {
	new_test_ext().execute_with(|| {
		let seller = 1;
		let buyer = 2;

		let _ = <Balances as Currency<_>>::deposit_creating(&seller, 100);
		let _ = <Balances as Currency<_>>::deposit_creating(&buyer, 100);

		let _ = KittiesModule::create(Origin::signed(seller));
		assert_eq!(<Balances as Currency<_>>::free_balance(&seller), 90);

		assert_noop!(
			KittiesModule::buy(Origin::signed(buyer), 3), // try to buy a non-existent kitty
			Error::<Test>::NoSuchKittyIndex
		);
	});
}

#[test]
fn buy_kitty_fails_if_no_such_owner() {
	new_test_ext().execute_with(|| {
		let seller = 1;
		let buyer = 2;

		let _ = <Balances as Currency<_>>::deposit_creating(&seller, 100);
		let _ = <Balances as Currency<_>>::deposit_creating(&buyer, 100);

		let _ = KittiesModule::create(Origin::signed(seller));
		assert_eq!(<Balances as Currency<_>>::free_balance(&seller), 90);

		// intentionally remove the owner of the created kitty
		KittiesModule::clear_owner();

		assert_noop!(KittiesModule::buy(Origin::signed(buyer), 1), Error::<Test>::NoSuchOwner);
	});
}

#[test]
fn buy_kitty_works_with_chain_transfer() {
	new_test_ext().execute_with(|| {
		let seller = 1;
		let buyer1 = 2;
		let buyer2 = 3;

		let _ = <Balances as Currency<_>>::deposit_creating(&seller, 100);
		let _ = <Balances as Currency<_>>::deposit_creating(&buyer1, 100);
		let _ = <Balances as Currency<_>>::deposit_creating(&buyer2, 100);

		let _ = KittiesModule::create(Origin::signed(seller));
		assert_eq!(<Balances as Currency<_>>::free_balance(&seller), 90);

		// seller puts the kitty on sale
		let _ = KittiesModule::sell(Origin::signed(seller), 1, 10);

		// buyer1 buys it
		assert_ok!(KittiesModule::buy(Origin::signed(buyer1), 1));
		assert_eq!(<Balances as Currency<_>>::free_balance(&seller), 110);
		assert_eq!(<Balances as Currency<_>>::free_balance(&buyer1), 90);

		// buyer1 puts it on sale again
		let _ = KittiesModule::sell(Origin::signed(buyer1), 1, 20);

		// buyer2 buys it
		assert_ok!(KittiesModule::buy(Origin::signed(buyer2), 1));

		// check balances:
		// seller's balance should be unchanged (unreserve only done once)
		assert_eq!(<Balances as Currency<_>>::free_balance(&seller), 110);
		// buyer1's balance: 100 - 10 + 20
		assert_eq!(<Balances as Currency<_>>::free_balance(&buyer1), 110);
		// buyer2's balance: 100 - 20
		assert_eq!(<Balances as Currency<_>>::free_balance(&buyer2), 80);
	});
}
