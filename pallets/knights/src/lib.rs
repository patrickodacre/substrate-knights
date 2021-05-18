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
    // thx to macro magic, we get to directly call this trait function
    use sp_io::hashing::blake2_128;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: pallet_balances::Config + frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
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
        pub gen: u64,
    }

    #[pallet::storage]
    #[pallet::getter(fn thing)]
    pub type Thing<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn knight_count)]
    pub type KnightCount<T: Config> = StorageValue<_, u64, OptionQuery>;

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
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<u64>, OptionQuery>;

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
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Knight Count Overflow
        KnightCountOverflow,
        KnightNotFound,
        KnightAlreadyExists,
        NotRightfulOwner,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(3, 4))]
        pub fn transfer_knight(
            origin: OriginFor<T>,
            id: u64,
            to: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let owner = KnightToOwner::<T>::get(&id).ok_or(Error::<T>::KnightNotFound)?;
            ensure!(owner == who, Error::<T>::NotRightfulOwner);

            // you could argue this check really isn't needed;
            // nevertheless, if we did want to check, we'd do it
            // before writing to storage below.
            if let Some(knight_ids) = OwnerToKnights::<T>::get(&to) {
                match knight_ids.iter().position(|&k_id| k_id == id) {
                    Some(_pos) => {
                        return Err(Error::<T>::KnightAlreadyExists)?;
                    }
                    _ => {}
                }
            }

            KnightToOwner::<T>::remove(id);
            KnightToOwner::<T>::insert(id, &to);

            let knight_id = OwnerToKnights::<T>::mutate(&owner, |ids| {
                // mutable reference
                let pos = ids
                    .as_ref()
                    .unwrap()
                    .binary_search_by(|probe| probe.cmp(&id))
                    .expect("Knight not found. Perhaps it was already transferred.");

                let removed_knight_id = ids.as_mut().unwrap().remove(pos);

                removed_knight_id
            });

            OwnerToKnights::<T>::append(&to, knight_id);

            Self::deposit_event(Event::KnightTransferred(id, who, to));

            Ok(().into())
        }

        /// An example dispatchable that takes a singles value as a parameter, writes the value to
        /// storage and emits an event. This function must be dispatched by a signed extrinsic.
        #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,4))]
        pub fn create_knight(origin: OriginFor<T>, name: Vec<u8>) -> DispatchResultWithPostInfo {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://substrate.dev/docs/en/knowledgebase/runtime/origin
            let who = ensure_signed(origin)?;

            let latest_id = KnightCount::<T>::get().unwrap_or(0);

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

            // Emit an event.
            Self::deposit_event(Event::KnightCreated(knight.id, owner));

            Ok(())
        }
    }
}
