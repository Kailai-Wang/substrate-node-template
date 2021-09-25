#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[cfg(test)]
pub mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::traits::Zero,
		traits::{Randomness, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use sp_io::hashing::blake2_128;
	use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, StaticLookup};

	#[derive(Clone, Encode, Decode, Default, PartialEq)]
	pub struct Kitty<T: Config> {
		pub index: T::KittyIndex,
		dna: [u8; 16],
		pub price: T::Balance,
		pub is_for_sale: bool,
		pub creator: T::AccountId, /* note down the creator so that we can unreserve the balance if need */
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: pallet_balances::Config + frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		// randomness used for random_seed()
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		// kitty index type which can be configured at runtime
		type KittyIndex: Default
			+ Copy
			+ core::fmt::Debug
			+ codec::FullCodec
			+ CheckedAdd
			+ AtLeast32BitUnsigned
			+ From<u64>;

		// amount of balance to reserve when creating a kitty (affects both `create` and `breed`)
		#[pallet::constant]
		type ReservedBalanceWhenCreate: Get<Self::Balance>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// store current kitty index to storage
	#[pallet::storage]
	#[pallet::getter(fn current_kitty_index)]
	pub(super) type CurrentKittyIndex<T: Config> = StorageValue<_, T::KittyIndex, ValueQuery>;

	// map that store all the kitties, with `KittyIndex` as key
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub type Kitties<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<Kitty<T>>, ValueQuery>;

	// map that store the kitty owners, with `KittyIndex` as key
	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T: Config> =
		StorageMap<_, Blake2_128Concat, T::KittyIndex, Option<T::AccountId>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// Event when a kitty gets created, [owner, kitty_index]
		KittyCreated(T::AccountId, T::KittyIndex),
		// Event when a kitty gets transferred, [sender, receiver, kitty_index]
		KittyTransferred(T::AccountId, T::AccountId, T::KittyIndex),
		// Event when a kitty gets bred, [owner, kitty_index]
		KittyBred(T::AccountId, T::KittyIndex),
		// Event when a kitty was put on sale, [kitty_index, price]
		KittyOnSale(T::KittyIndex, T::Balance),
		// Event whena kitty was bought, [kitty_index, new_owner]
		KittyBought(T::KittyIndex, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		// Error kitty index overflows when trying to get a new index
		KittyIndexOverflow,
		// Error when the account is not the kitty owner
		NotKittyOwner,
		// Error when both parents have same kitty index when breeding
		SameParentIndex,
		// Error when kitty index doesn't exist
		NoSuchKittyIndex,
		// Error when sale price is invalid when selling a kitty
		InvalidSellPrice,
		// Error when owner can't be found when buying a kitty
		NoSuchOwner,
		// Error when kitty is not for sale
		KittyNotForSale,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// create a kitty, set the owner to the `origin`
		#[pallet::weight(100)]
		pub fn create(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let dna = Self::random_hash(&sender);
			let kitty_index = Self::create_kitty_internal(sender.clone(), dna)?;

			Self::deposit_event(Event::<T>::KittyCreated(sender.clone(), kitty_index));

			Ok(().into())
		}

		// transfer a kitty with `kitty_index` from `origin` to `to`
		#[pallet::weight(100)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			kitty_index: T::KittyIndex,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// only kitty owner can transfer it
			ensure!(
				Some(sender.clone()) == Owner::<T>::get(kitty_index),
				Error::<T>::NotKittyOwner
			);

			Self::transfer_internal(kitty_index, sender.clone(), to.clone());

			Self::deposit_event(Event::<T>::KittyTransferred(
				sender.clone(),
				to.clone(),
				kitty_index,
			));

			Ok(().into())
		}

		// breed a kitty out of `kitty_index_1` and `kitty_index_2`, mutate their dna randomly
		#[pallet::weight(100)]
		pub fn breed(
			origin: OriginFor<T>,
			kitty_index_1: T::KittyIndex,
			kitty_index_2: T::KittyIndex,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// `kitty_index_1` and `kitty_index_2` must be different
			ensure!(kitty_index_1 != kitty_index_2, Error::<T>::SameParentIndex);

			let kitty1 = Kitties::<T>::get(kitty_index_1).ok_or(Error::<T>::NoSuchKittyIndex)?;
			let kitty2 = Kitties::<T>::get(kitty_index_2).ok_or(Error::<T>::NoSuchKittyIndex)?;

			let selector = Self::random_hash(&sender);
			let dna1 = kitty1.dna;
			let dna2 = kitty2.dna;
			let mut new_dna = [0u8; 16];

			// sanity check: the length of DNA should be equal,
			// otherwise we might have problems with index access
			ensure!(dna1.len() == dna2.len(), "DNA length is not equal");

			// sanity check: the length of selector must match,
			// otherwise we might have problems with index access
			ensure!(dna1.len() == selector.len(), "selector length does not match");

			for i in 0..dna1.len() {
				new_dna[i] = (selector[i] & dna1[i]) | (!selector[i] & dna2[i]);
			}

			let kitty_index = Self::create_kitty_internal(sender.clone(), new_dna)?;

			Self::deposit_event(Event::<T>::KittyBred(sender.clone(), kitty_index));

			Ok(().into())
		}

		// put the kitty with `kitty_index` on sale with the given `price`
		#[pallet::weight(100)]
		pub fn sell(
			origin: OriginFor<T>,
			kitty_index: T::KittyIndex,
			new_price: T::Balance,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// only kitty owner can put it on sale
			ensure!(
				Some(sender.clone()) == Owner::<T>::get(kitty_index),
				Error::<T>::NotKittyOwner
			);

			ensure!(!new_price.is_zero(), Error::<T>::InvalidSellPrice);

			let mut kitty = Kitties::<T>::get(kitty_index).ok_or(Error::<T>::NoSuchKittyIndex)?;
			kitty.is_for_sale = true;
			kitty.price = new_price;

			Kitties::<T>::insert(kitty_index, Some(kitty));

			Self::deposit_event(Event::<T>::KittyOnSale(kitty_index, new_price));

			Ok(().into())
		}

		// buy the kitty with `kitty_index`
		#[pallet::weight(100)]
		pub fn buy(origin: OriginFor<T>, kitty_index: T::KittyIndex) -> DispatchResultWithPostInfo {
			let buyer = ensure_signed(origin.clone())?;

			// check if Kitty exists
			let mut kitty = Kitties::<T>::get(kitty_index).ok_or(Error::<T>::NoSuchKittyIndex)?;
			// check if the Kitty has owner
			let owner = Owner::<T>::get(kitty_index).ok_or(Error::<T>::NoSuchOwner)?;

			ensure!(kitty.is_for_sale, Error::<T>::KittyNotForSale);

			// try to transfer the balance
			let _ = pallet_balances::Pallet::<T>::transfer(
				origin,
				T::Lookup::unlookup(owner.clone()),
				kitty.price,
			)?;

			Self::transfer_internal(kitty_index, owner.clone(), buyer.clone());

			// update kitty is_for_sale status
			kitty.is_for_sale = false;
			Kitties::<T>::insert(kitty_index, Some(kitty));

			// emit the event
			Self::deposit_event(Event::<T>::KittyBought(kitty_index, buyer.clone()));
			Ok(().into())
		}
	}

	// pallet impl that are mostly private functions that can be called by dispatchables
	impl<T: Config> Pallet<T> {
		// set kitty index for testing purpose
		#[cfg(test)]
		pub fn set_kitty_index(kitty_index: T::KittyIndex) {
			CurrentKittyIndex::<T>::put(kitty_index);
		}

		// clear owner storage for testing purpose
		#[cfg(test)]
		pub fn clear_owner() {
			Owner::<T>::remove_all(None);
		}

		// get a random hash out of a T::AccountId
		fn random_hash(sender: &T::AccountId) -> [u8; 16] {
			let payload = (
				T::Randomness::random_seed(),
				&sender,
				frame_system::Pallet::<T>::extrinsic_index(),
			);

			payload.using_encoded(blake2_128)
		}

		// increment the current stored kitty index by 1 and return the new value if succeeds
		fn increment_kitty_index() -> Result<T::KittyIndex, sp_runtime::DispatchError> {
			CurrentKittyIndex::<T>::try_mutate(|index| {
				let next = index.checked_add(&1u64.into()).ok_or(Error::<T>::KittyIndexOverflow)?;
				*index = next;
				Ok(next)
			})
		}

		// create a kitty with provided owner and dna info
		// act as a common fn that can be called by `create` and `breed`
		fn create_kitty_internal(
			owner: T::AccountId,
			dna: [u8; 16],
		) -> Result<T::KittyIndex, sp_runtime::DispatchError> {
			let kitty_index = Self::increment_kitty_index()?;
			let new_kitty = Kitty::<T> {
				index: kitty_index,
				dna,
				price: 0u8.into(),
				is_for_sale: false,
				creator: owner.clone(),
			};

			// try to reserve some amount when creating
			let _ = <pallet_balances::Pallet<T> as ReservableCurrency<_>>::reserve(
				&owner,
				T::ReservedBalanceWhenCreate::get(),
			)?;

			Kitties::<T>::insert(kitty_index, Some(new_kitty));
			Owner::<T>::insert(kitty_index, Some(owner.clone()));

			Ok(kitty_index)
		}

		// transfer the ownership of a kitty with `kitty_index` from `from` to `to`
		// meanwhile unreserve the balance of `from` when he
		fn transfer_internal(kitty_index: T::KittyIndex, from: T::AccountId, to: T::AccountId) {
			let owner = Owner::<T>::get(kitty_index).unwrap();

			// unreserve the balance if `from` is the creator of this kitty
			if from == owner {
				<pallet_balances::Pallet<T> as ReservableCurrency<_>>::unreserve(
					&from,
					T::ReservedBalanceWhenCreate::get(),
				);
			}
			Owner::<T>::insert(kitty_index, Some(to));
		}
	}
}
