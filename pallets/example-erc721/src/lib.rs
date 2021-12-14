#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use sp_core::U256;
use sp_std::prelude::*;

pub type TokenId = U256;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, scale_info::TypeInfo)]
pub struct Erc721Token {
    pub id: TokenId,
    pub metadata: Vec<u8>,
}

#[frame_support::pallet]
pub mod pallet {

    use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        
        type Identifier: Get<[u8; 32]>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn tokens)]
    pub(super) type Tokens<T> = StorageMap<_, Blake2_128Concat, TokenId, Option<Erc721Token>, ValueQuery>;


    #[pallet::storage]
    #[pallet::getter(fn owner_of)]
    pub(super) type TokenOwner<T:Config> = StorageMap<_, Blake2_128Concat, TokenId, Option<T::AccountId>, ValueQuery>;

    #[pallet::type_value]
    pub(super) fn MyDefault() -> U256 { U256::zero()}

    #[pallet::storage]
	#[pallet::getter(fn token_count)]
	pub(super) type TokenCount<T> = StorageValue<_, U256, ValueQuery,MyDefault>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
        /// New token created
        Minted(T::AccountId, TokenId),
        /// Token transfer between two parties
        Transferred(T::AccountId, T::AccountId, TokenId),
        /// Token removed from the system
        Burned(TokenId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
        /// ID not recognized
        TokenIdDoesNotExist,
        /// Already exists with an owner
        TokenAlreadyExists,
        /// Origin is not owner
        NotOwner,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

               /// Creates a new token with the given token ID and metadata, and gives ownership to owner
        #[pallet::weight(195_000_000)]
        pub fn mint(origin:OriginFor<T>, owner: T::AccountId, id: TokenId, metadata: Vec<u8>) -> DispatchResult {
            ensure_root(origin)?;

            Self::mint_token(owner, id, metadata)?;

            Ok(())
        }

        /// Changes ownership of a token sender owns
        #[pallet::weight(195_000_000)]
        pub fn transfer(origin:OriginFor<T>, to: T::AccountId, id: TokenId) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            Self::transfer_from(sender, to, id)?;

            Ok(())
        }

        /// Remove token from the system
        #[pallet::weight(195_000_000)]
        pub fn burn(origin:OriginFor<T>, id: TokenId) -> DispatchResult {
            ensure_root(origin)?;

            let owner = Self::owner_of(id).ok_or(Error::<T>::TokenIdDoesNotExist)?;

            Self::burn_token(owner, id)?;

            Ok(())
        }

	}
}


impl<T: Config> Pallet<T> {
    /// Creates a new token in the system.
    pub fn mint_token(owner: T::AccountId, id: TokenId, metadata: Vec<u8>) -> DispatchResult {
        ensure!(!Tokens::<T>::contains_key(id), Error::<T>::TokenAlreadyExists);

        let new_token = Erc721Token { id, metadata };

        <Tokens<T>>::insert(&id, Some(new_token));
        <TokenOwner<T>>::insert(&id, Some(owner.clone()));
        let new_total = <TokenCount<T>>::get().saturating_add(U256::one());
        <TokenCount<T>>::put(new_total);

        Self::deposit_event(Event::Minted(owner, id));

        Ok(())
    }

    /// Modifies ownership of a token
    pub fn transfer_from(from: T::AccountId, to: T::AccountId, id: TokenId) -> DispatchResult {
        // Check from is owner and token exists
        let owner = Self::owner_of(id).ok_or(Error::<T>::TokenIdDoesNotExist)?;
        ensure!(owner == from, Error::<T>::NotOwner);
        // Update owner
        <TokenOwner<T>>::insert(&id, Some(to.clone()));

        Self::deposit_event(Event::Transferred(from, to, id));

        Ok(())
    }

    /// Deletes a token from the system.
    pub fn burn_token(from: T::AccountId, id: TokenId) -> DispatchResult {
        let owner = Self::owner_of(id).ok_or(Error::<T>::TokenIdDoesNotExist)?;
        ensure!(owner == from, Error::<T>::NotOwner);

        <Tokens<T>>::remove(&id);
        <TokenOwner<T>>::remove(&id);
        let new_total = <TokenCount<T>>::get().saturating_sub(U256::one());
        <TokenCount<T>>::put(new_total);

        Self::deposit_event(Event::Burned(id));

        Ok(())
    }
}


