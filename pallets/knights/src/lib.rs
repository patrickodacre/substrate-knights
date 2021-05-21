#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::traits::Vec;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use serde::{Deserialize, Serialize};

    use frame_support::traits::Currency;
    use sp_runtime::traits::Zero;

    // thx to macro magic, we get to directly call this trait function
    use sp_io::hashing::blake2_128;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: pallet_balances::Config + frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        // type Currency: Currency<<Self as frame_system::Config>::AccountId>;
        // or...
        type Currency: Currency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[derive(Encode, Decode, Deserialize, Serialize, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(Debug))]
    pub struct Knight<Balance> {
        pub id: u64,
        pub dna: [u8; 16],
        pub name: Vec<u8>,
        pub wealth: Balance,
        pub price: Balance,
        pub gen: u64,
    }

    #[pallet::storage]
    #[pallet::getter(fn thing)]
    pub type Thing<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn knight_count)]
    pub type KnightCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn knights)]
    pub type Knights<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, Knight<T::Balance>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn knight_to_owner)]
    pub type KnightToOwner<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, T::AccountId, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn owner_to_knights)]
    pub type OwnerToKnights<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<u64>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn owner_to_knight_count)]
    pub type OwnerToKnightCount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;

    // The genesis config type.
    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub thing: u64,
        // pub bar: Vec<(T::AccountId, T::Balance)>,
        pub knights: Vec<(u64, Knight<T::Balance>)>,
    }

    // The default value for the genesis config type.
    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                thing: Default::default(),
                // bar: Default::default(),
                knights: Default::default(),
            }
        }
    }

    // The build of genesis for the pallet.
    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            <Thing<T>>::put(&self.thing);
            for (id, account_id) in &self.knights {
                let knight = Knight {
                    id: 1,
                    name: "OriginKnight".as_bytes().to_vec(),
                    dna: (1).using_encoded(blake2_128),
                    wealth: 0u8.into(),
                    price: 0u8.into(),
                    gen: 0,
                };

                Knights::<T>::insert(id, knight);
            }
        }
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        KnightCreated(u64, T::AccountId),
        /// [knight_id, from_account_id, to_account_id]
        KnightTransferred(u64, T::AccountId, T::AccountId),
        /// [knight_id, price]
        KnightPriceSet(u64, T::Balance),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Knight Count Overflow
        KnightCountOverflow,
        OwnerToKnightCountOverflow,
        OwnerToKnightCountUnderflow,
        KnightNotFound,
        KnightAlreadyExists,
        NotRightfulOwner,
        KnightTransferFailed,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(4, 6))]
        pub fn transfer_knight(
            origin: OriginFor<T>,
            id: u64,
            to: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let from = ensure_signed(origin)?;

            let owner = KnightToOwner::<T>::get(&id).ok_or(Error::<T>::KnightNotFound)?;
            ensure!(owner == from, Error::<T>::NotRightfulOwner);

            Self::_transfer_knight(id, from, to).expect("Transfers Knight");

            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,1))]
        pub fn set_price(
            origin: OriginFor<T>,
            knight_id: u64,
            price: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let mut knight = Knights::<T>::get(knight_id).ok_or(Error::<T>::KnightNotFound)?;
            let owner = KnightToOwner::<T>::get(&knight_id).ok_or(Error::<T>::KnightNotFound)?;

            ensure!(owner == who, Error::<T>::NotRightfulOwner);

            knight.price = price;

            Knights::<T>::insert(knight_id, knight);

            Self::deposit_event(Event::KnightPriceSet(knight_id, price));

            Ok(().into())
        }

        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(6,8))]
        pub fn buy_knight(origin: OriginFor<T>, knight_id: u64) -> DispatchResultWithPostInfo {
            let buyer = ensure_signed(origin)?;

            // Before we send our tokens we have to make sure
            // our transfer of the knight won't fail. If it were to fail
            // the owner / seller of the knight would still have the tokens.

            // the knight exists
            let mut knight = Knights::<T>::get(knight_id).ok_or(Error::<T>::KnightNotFound)?;

            ensure!(
                !knight.price.is_zero(),
                "The knight you want to buy isn't for sale."
            );

            // we'll send funds to the owner of the knight
            let owner = KnightToOwner::<T>::get(knight_id).ok_or(Error::<T>::KnightNotFound)?;

            ensure!(owner != buyer, "You already own this Knight");

            <pallet_balances::Pallet<T> as Currency<_>>::transfer(
                &buyer,
                &owner,
                knight.price,
                frame_support::traits::ExistenceRequirement::KeepAlive,
            )?;

            // NOTE on underflow and overflow::
            // Since these counts are set when a Knight is
            // minted if a user owns a Knight, then his count
            // will always be >= 1.
            // as for the buyer, his knight count will never exceed
            // the total number of knights minted. Since they are
            // both u64, we can be certain that an overflow will never occur.
            // All that said, it's typical to see .expect() statements
            // in Substrate code to document why something will never fail.
            Self::_transfer_knight(knight_id, owner, buyer).expect("Transfers Knight");

            // update price to zero so this Knight cannot be purchased again
            // until the new owner decides.
            knight.price = T::Balance::zero();
            Knights::<T>::insert(knight_id, &knight);

            Ok(().into())
        }

        /// An example dispatchable that takes a singles value as a parameter, writes the value to
        /// storage and emits an event. This function must be dispatched by a signed extrinsic.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2,5))]
        pub fn create_knight(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let who = ensure_signed(origin)?;

            let latest_id = KnightCount::<T>::get();

            let new_id = latest_id
                .checked_add(1)
                .ok_or(Error::<T>::KnightCountOverflow)?;

            // NOTE:: how to test this?
            ensure!(
                !Knights::<T>::contains_key(new_id),
                Error::<T>::KnightAlreadyExists
            );

            let knight = Knight {
                id: new_id,
                name,
                dna: (new_id, &who).using_encoded(blake2_128),
                wealth: 0u8.into(),
                price: 0u8.into(),
                gen: 0,
            };

            Self::_mint(who, knight)?;

            return Ok(().into());
        }
    }

    impl<T: Config> Pallet<T> {
        fn _mint(owner: T::AccountId, knight: Knight<T::Balance>) -> Result<(), &'static str> {
            Knights::<T>::insert(knight.id, &knight);
            KnightCount::<T>::put(knight.id);
            KnightToOwner::<T>::insert(knight.id, &owner);
            OwnerToKnights::<T>::append(&owner, knight.id);

            let current_owner_to_knight_count = OwnerToKnightCount::<T>::get(&owner);
            let new_count = current_owner_to_knight_count
                .checked_add(1)
                .ok_or(Error::<T>::OwnerToKnightCountOverflow)?;

            OwnerToKnightCount::<T>::insert(&owner, new_count);

            // Emit an event.
            Self::deposit_event(Event::KnightCreated(knight.id, owner));

            Ok(())
        }

        fn _transfer_knight(
            knight_id: u64,
            from: T::AccountId,
            to: T::AccountId,
        ) -> Result<(), DispatchError> {
            // you could argue this check really isn't needed;
            // nevertheless, if we did want to check, we'd do it
            // before writing to storage below.
            match OwnerToKnights::<T>::get(&to)
                .iter()
                .position(|&k_id| k_id == knight_id)
            {
                Some(_pos) => {
                    return Err(Error::<T>::KnightAlreadyExists)?;
                }
                _ => {}
            }

            KnightToOwner::<T>::remove(knight_id);
            KnightToOwner::<T>::insert(knight_id, &to);

            // remove the knight_id from owner's list of knight ids
            OwnerToKnights::<T>::mutate(&from, |ids| {
                // mutable reference
                let pos = ids
                    .binary_search_by(|probe| probe.cmp(&knight_id))
                    .expect("Knight not found. Perhaps it was already transferred.");

                ids.remove(pos);
            });

            OwnerToKnights::<T>::append(&to, knight_id);

            // these underflow / overflows aren't possible,
            // so at the call site of this function, we use an .expect()
            // to document why this function will never fail.
            let from_count = OwnerToKnightCount::<T>::get(&from);
            let new_from_count = from_count
                .checked_sub(1)
                .ok_or(Error::<T>::OwnerToKnightCountUnderflow)?;
            OwnerToKnightCount::<T>::insert(&from, new_from_count);

            let to_count = OwnerToKnightCount::<T>::get(&to);
            let new_to_count = to_count
                .checked_add(1)
                .ok_or(Error::<T>::OwnerToKnightCountOverflow)?;
            OwnerToKnightCount::<T>::insert(&to, new_to_count);

            Self::deposit_event(Event::KnightTransferred(knight_id, from, to));

            Ok(())
        }
    }
}
