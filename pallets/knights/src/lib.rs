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
    // thx to macro magic, we get to directly call this trait function
    use sp_io::hashing::blake2_128;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[derive(Encode, Decode, Clone, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(Debug))]
    pub struct Knight {
        pub id: [u8; 16],
        pub name: Vec<u8>,
    }

    #[pallet::storage]
    #[pallet::getter(fn knight_count)]
    pub type KnightCount<T: Config> = StorageValue<_, u64>;

    #[pallet::storage]
    #[pallet::getter(fn knights)]
    pub type Knights<T: Config> = StorageMap<_, Blake2_128Concat, u64, Knight>;

    #[pallet::storage]
    #[pallet::getter(fn knight_to_owner)]
    pub type KnightToOwner<T: Config> = StorageMap<_, Blake2_128Concat, u64, T::AccountId>;

    #[pallet::storage]
    #[pallet::getter(fn owner_to_knights)]
    pub type OwnerToKnights<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<u64>>;

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
        NotRightfulOwner,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn transfer_knight(
            origin: OriginFor<T>,
            id: u64,
            to: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            if let Some(owner) = KnightToOwner::<T>::get(&id) {
                ensure!(owner == who, Error::<T>::NotRightfulOwner);
            } else {
                return Err(Error::<T>::KnightNotFound)?;
            }

            KnightToOwner::<T>::remove(id);
            KnightToOwner::<T>::insert(id, &to);

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

            let id = latest_id
                .checked_add(1)
                .ok_or(Error::<T>::KnightCountOverflow)?;

            let knight_id = (id, &who).using_encoded(blake2_128);

            // NOTE:: how to test this?
            ensure!(!Knights::<T>::contains_key(id), "This id already exists");

            let k = Knight {
                name,
                id: knight_id,
            };

            Knights::<T>::insert(id, k);
            KnightCount::<T>::put(id);

            KnightToOwner::<T>::insert(id, &who);
            OwnerToKnights::<T>::append(&who, id);

            // Emit an event.
            Self::deposit_event(Event::KnightCreated(id, who));

            return Ok(().into());
        }
    }
}
