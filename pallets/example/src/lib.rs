#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use chainbridge;
use erc721;

use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_support::traits::{Currency, EnsureOrigin, ExistenceRequirement::AllowDeath, Get};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::SaturatedConversion;
use sp_core::U256;
use sp_std::prelude::*;
use sp_std::convert::From;



pub type ResourceId = chainbridge::ResourceId;

pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;


#[frame_support::pallet]
pub mod pallet {

    use super::*;
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + chainbridge::Config + erc721::Config{

        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Specifies the origin check provided by the bridge for calls that can only be called by the bridge pallet
        type BridgeOrigin: EnsureOrigin<Self::Origin, Success = Self::AccountId>;
    
        /// The currency mechanism.
        type Currency: Currency<Self::AccountId>;
    
        /// Ids can be defined by the runtime and passed in, perhaps from blake2b_128 hashes.
        type HashId: Get<ResourceId>;
        type NativeTokenId: Get<ResourceId>;
        type Erc721Id: Get<ResourceId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
        Remark(T::Hash),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
        InvalidTransfer,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

        /// Transfers an arbitrary hash to a (whitelisted) destination chain.
        #[pallet::weight(195_000_000)]
        pub fn transfer_hash(origin:OriginFor<T>, hash: T::Hash, dest_id: chainbridge::ChainId) -> DispatchResult {
            ensure_signed(origin)?;

            let resource_id = T::HashId::get();
            let metadata: Vec<u8> = hash.as_ref().to_vec();
            <chainbridge::Pallet<T>>::transfer_generic(dest_id, resource_id, metadata)
        }

        /// Transfers some amount of the native token to some recipient on a (whitelisted) destination chain.
        #[pallet::weight(195_000_000)]
        pub fn transfer_native(origin: OriginFor<T>, amount: BalanceOf<T>, recipient: Vec<u8>, dest_id: chainbridge::ChainId) -> DispatchResult {
            let source = ensure_signed(origin)?;
            ensure!(<chainbridge::Pallet<T>>::chain_whitelisted(dest_id), Error::<T>::InvalidTransfer);
            let bridge_id = <chainbridge::Pallet<T>>::account_id();
            T::Currency::transfer(&source, &bridge_id, amount.into(), AllowDeath)?;

            let resource_id = T::NativeTokenId::get();
            <chainbridge::Pallet<T>>::transfer_fungible(dest_id, resource_id, recipient, U256::from(amount.saturated_into::<u128>()))
        }

                /// Transfer a non-fungible token (erc721) to a (whitelisted) destination chain.
        #[pallet::weight(195_000_000)]
        pub fn transfer_erc721(origin:OriginFor<T>, recipient: Vec<u8>, token_id: U256, dest_id: chainbridge::ChainId) -> DispatchResult {
            let source = ensure_signed(origin)?;
            ensure!(<chainbridge::Pallet<T>>::chain_whitelisted(dest_id), Error::<T>::InvalidTransfer);
            match <erc721::Pallet<T>>::tokens(&token_id) {
                Some(token) => {
                    <erc721::Pallet<T>>::burn_token(source, token_id)?;
                    let resource_id = T::Erc721Id::get();
                    let tid: &mut [u8] = &mut[0; 32];
                    token_id.to_big_endian(tid);
                    <chainbridge::Pallet<T>>::transfer_nonfungible(dest_id, resource_id, tid.to_vec(), recipient, token.metadata)
                }
                None => Err(Error::<T>::InvalidTransfer)?
            }
        }

        /// Executes a simple currency transfer using the bridge account as the source
        #[pallet::weight(195_000_000)]
        pub fn transfer(origin: OriginFor<T>, to: T::AccountId, amount: BalanceOf<T>, r_id: ResourceId) -> DispatchResult {
            let source = T::BridgeOrigin::ensure_origin(origin)?;
            <T as Config>::Currency::transfer(&source, &to, amount.into(), AllowDeath)?;
            Ok(())
        }

        /// This can be called by the bridge to demonstrate an arbitrary call from a proposal.
        #[pallet::weight(195_000_000)]
        pub fn remark(origin: OriginFor<T>, hash: T::Hash, r_id: ResourceId) -> DispatchResult {
            T::BridgeOrigin::ensure_origin(origin)?;
            Self::deposit_event(Event::Remark(hash));
            Ok(())
        }

        /// Allows the bridge to issue new erc721 tokens
        #[pallet::weight(195_000_000)]
        pub fn mint_erc721(origin: OriginFor<T>, recipient: T::AccountId, id: U256, metadata: Vec<u8>, r_id: ResourceId) -> DispatchResult {
            T::BridgeOrigin::ensure_origin(origin)?;
            <erc721::Pallet<T>>::mint_token(recipient, id, metadata)?;
            Ok(())
        }

	}
}

