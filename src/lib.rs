//! # Unique Assets Implementation
//!
//! This pallet exposes capabilities for managing unique assets, also known as
//! non-fungible tokens (NFTs).
//!
//! - [`nft::Trait`](./trait.Trait.html)
//! - [`Calls`](./enum.Call.html)
//! - [`Errors`](./enum.Error.html)
//! - [`Events`](./enum.RawEvent.html)
//!
//! ## Overview
//!
//! Assets that share a common metadata structure may be created and distributed
//! by an asset admin. Asset owners may burn assets or transfer their
//! ownership. Configuration parameters are used to limit the total number of a
//! type of asset that may exist as well as the number that any one account may
//! own. Assets are uniquely identified by the hash of the info that defines
//! them, as calculated by the runtime system's hashing algorithm.
//!
//! This pallet implements the [`UniqueAssets`](./nft/trait.UniqueAssets.html)
//! trait.
//!
//! ### Dispatchable Functions
//!
//! * [`mint`](./enum.Call.html#variant.mint) - Use the provided asset info to
//!   create a new unique asset for the specified user. May only be called by
//!   the asset admin.
//!
//! * [`burn`](./enum.Call.html#variant.burn) - Destroy an asset. May only be
//!   called by asset owner.
//!
//! * [`transfer`](./enum.Call.html#variant.transfer) - Transfer ownership of
//!   an asset to another account. May only be called by current asset owner.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, FullCodec};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,
    traits::{EnsureOrigin, Get},
    Hashable,
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::{
    traits::{Hash, Member},
    RuntimeDebug,
};
use sp_std::{
    cmp::{Eq, Ordering},
    fmt::Debug,
    vec::Vec,
};

pub mod nft;
pub use crate::nft::{UniqueAssets, NFT};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait<I = DefaultInstance>: system::Trait {
    /// The dispatch origin that is able to mint new instances of this type of asset.
    type AssetAdmin: EnsureOrigin<Self::Origin>;
    /// The data type that is used to describe this type of asset.
    type AssetInfo: Hashable + Member + Debug + Default + FullCodec;
    /// The maximum number of this type of asset that may exist (minted - burned).
    type AssetLimit: Get<u128>;
    /// The maximum number of this type of asset that any single account may own.
    type UserAssetLimit: Get<u64>;
    type Event: From<Event<Self, I>> + Into<<Self as system::Trait>::Event>;
}

/// The runtime system's hashing algorithm is used to uniquely identify assets.
pub type AssetId<T> = <T as frame_system::Trait>::Hash;

/// An alias for this pallet's NFT implementation.
pub type IdentifiedAssetFor<T, I> = IdentifiedAsset<AssetId<T>, <T as Trait<I>>::AssetInfo>;

/// A generic definition of an NFT that will be used by this pallet.
#[derive(Encode, Decode, Clone, Eq, RuntimeDebug)]
pub struct IdentifiedAsset<Hash, AssetInfo> {
    pub id: Hash,
    pub asset: AssetInfo,
}

impl<AssetId: Ord, AssetInfo: Eq> Ord for IdentifiedAsset<AssetId, AssetInfo> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<AssetId: Ord, AssetInfo> PartialOrd for IdentifiedAsset<AssetId, AssetInfo> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl<AssetId: Eq, AssetInfo> PartialEq for IdentifiedAsset<AssetId, AssetInfo> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<AssetId, AssetInfo> NFT for IdentifiedAsset<AssetId, AssetInfo> {
    type Id = AssetId;
    type Info = AssetInfo;
}

decl_storage! {
    trait Store for Module<T: Trait<I>, I: Instance = DefaultInstance> as NFT {
        /// The total number of this type of asset that exists (minted - burned).
        Total get(fn total): u128 = 0;
        /// The total number of this type of asset that has been burned (may overflow).
        Burned get(fn burned): u128 = 0;
        /// The total number of this type of asset owned by an account.
        TotalForAccount get(fn total_for_account): map hasher(blake2_128_concat) T::AccountId => u64 = 0;
        /// A mapping from an account to a list of all of the assets of this type that are owned by it.
        AssetsForAccount get(fn assets_for_account): map hasher(blake2_128_concat) T::AccountId => Vec<IdentifiedAssetFor<T, I>>;
        /// A mapping from an asset ID to the account that owns it.
        AccountForAsset get(fn account_for_asset): map hasher(identity) AssetId<T> => T::AccountId;
    }
}

decl_event!(
    pub enum Event<T, I = DefaultInstance>
    where
        AssetId = <T as system::Trait>::Hash,
        AccountId = <T as system::Trait>::AccountId,
    {
        /// The asset has been burned.
        Burned(AssetId),
        /// The asset has been minted and distributed to the account.
        Minted(AssetId, AccountId),
        /// Ownership of the asset has been transferred to the account.
        Transferred(AssetId, AccountId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait<I>, I: Instance> {
        // Thrown when there is an attempt to mint a duplicate asset.
        AssetExists,
        // Thrown when there is an attempt to burn or transfer a nonexistent asset.
        NonexistentAsset,
        // Thrown when someone who is not the owner of an asset attempts to transfer or burn it.
        NotAssetOwner,
        // Thrown when the asset admin attempts to mint an asset and the maximum number of this
        // type of asset already exists.
        TooManyAssets,
        // Thrown when an attempt is made to mint or transfer an asset to an account that already
        // owns the maximum number of this type of asset.
        TooManyAssetsForAccount,
    }
}

decl_module! {
    pub struct Module<T: Trait<I>, I: Instance = DefaultInstance> for enum Call where origin: T::Origin {
        type Error = Error<T, I>;
        fn deposit_event() = default;

        /// Create a new unique asset from the provided asset info and identify the specified
        /// account as its owner. The ID of the new asset will be equal to the hash of the info
        /// that defines it, as calculated by the runtime system's hashing algorithm.
        ///
        /// The dispatch origin for this call must be the asset admin.
        ///
        /// This function will throw an error if it is called with asset info that describes
        /// an existing (duplicate) asset, if the maximum number of this type of asset already
        /// exists or if the specified owner already owns the maximum number of this type of
        /// asset.
        ///
        /// - `owner_account`: Receiver of the asset.
        /// - `asset_info`: The information that defines the asset.
        #[weight = 10_000]
        pub fn mint(origin, owner_account: T::AccountId, asset_info: T::AssetInfo) -> dispatch::DispatchResult {
            T::AssetAdmin::ensure_origin(origin)?;

            let asset_id = <Self as UniqueAssets<_>>::mint(&owner_account, asset_info)?;
            Self::deposit_event(RawEvent::Minted(asset_id, owner_account.clone()));
            Ok(())
        }

        /// Destroy the specified asset.
        ///
        /// The dispatch origin for this call must be the asset owner.
        ///
        /// - `asset_id`: The hash (calculated by the runtime system's hashing algorithm)
        ///   of the info that defines the asset to destroy.
        #[weight = 10_000]
        pub fn burn(origin, asset_id: AssetId<T>) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(who == Self::account_for_asset(&asset_id), Error::<T, I>::NotAssetOwner);

            <Self as UniqueAssets<_>>::burn(&asset_id)?;
            Self::deposit_event(RawEvent::Burned(asset_id.clone()));
            Ok(())
        }

        /// Transfer an asset to a new owner.
        ///
        /// The dispatch origin for this call must be the asset owner.
        ///
        /// This function will throw an error if the new owner already owns the maximum
        /// number of this type of asset.
        ///
        /// - `dest_account`: Receiver of the asset.
        /// - `asset_id`: The hash (calculated by the runtime system's hashing algorithm)
        ///   of the info that defines the asset to destroy.
        #[weight = 10_000]
        pub fn transfer(origin, dest_account: T::AccountId, asset_id: AssetId<T>) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(who == Self::account_for_asset(&asset_id), Error::<T, I>::NotAssetOwner);

            <Self as UniqueAssets<_>>::transfer(&dest_account, &asset_id)?;
            Self::deposit_event(RawEvent::Transferred(asset_id.clone(), dest_account.clone()));
            Ok(())
        }
    }
}

impl<T: Trait<I>, I: Instance> UniqueAssets<IdentifiedAsset<AssetId<T>, <T as Trait<I>>::AssetInfo>>
    for Module<T, I>
{
    type AccountId = <T as system::Trait>::AccountId;
    type AssetLimit = T::AssetLimit;
    type UserAssetLimit = T::UserAssetLimit;

    fn total() -> u128 {
        Self::total()
    }

    fn burned() -> u128 {
        Self::burned()
    }

    fn total_for_account(account: &T::AccountId) -> u64 {
        Self::total_for_account(account)
    }

    fn assets_for_account(
        account: &T::AccountId,
    ) -> Vec<IdentifiedAsset<AssetId<T>, <T as Trait<I>>::AssetInfo>> {
        Self::assets_for_account(account)
    }

    fn owner_of(asset_id: &AssetId<T>) -> T::AccountId {
        Self::account_for_asset(asset_id)
    }

    fn mint(
        owner_account: &T::AccountId,
        asset_info: <T as Trait<I>>::AssetInfo,
    ) -> dispatch::result::Result<AssetId<T>, dispatch::DispatchError> {
        let asset_id = T::Hashing::hash_of(&asset_info);

        ensure!(
            !AccountForAsset::<T, I>::contains_key(&asset_id),
            Error::<T, I>::AssetExists
        );

        ensure!(
            Self::total_for_account(owner_account) < T::UserAssetLimit::get(),
            Error::<T, I>::TooManyAssetsForAccount
        );

        ensure!(
            Self::total() < T::AssetLimit::get(),
            Error::<T, I>::TooManyAssets
        );

        let new_asset = IdentifiedAsset {
            id: asset_id,
            asset: asset_info,
        };

        Total::<I>::mutate(|total| *total += 1);
        TotalForAccount::<T, I>::mutate(owner_account, |total| *total += 1);
        AssetsForAccount::<T, I>::mutate(owner_account, |assets| {
            match assets.binary_search(&new_asset) {
                Ok(_pos) => {} // should never happen
                Err(pos) => assets.insert(pos, new_asset),
            }
        });
        AccountForAsset::<T, I>::insert(asset_id, &owner_account);

        Ok(asset_id)
    }

    fn burn(asset_id: &AssetId<T>) -> dispatch::DispatchResult {
        let owner = Self::owner_of(asset_id);
        ensure!(
            owner != T::AccountId::default(),
            Error::<T, I>::NonexistentAsset
        );

        let burn_asset = IdentifiedAsset::<AssetId<T>, <T as Trait<I>>::AssetInfo> {
            id: *asset_id,
            asset: <T as Trait<I>>::AssetInfo::default(),
        };

        Total::<I>::mutate(|total| *total -= 1);
        Burned::<I>::mutate(|total| *total += 1);
        TotalForAccount::<T, I>::mutate(&owner, |total| *total -= 1);
        AssetsForAccount::<T, I>::mutate(owner, |assets| {
            let pos = assets
                .binary_search(&burn_asset)
                .expect("We already checked that we have the correct owner; qed");
            assets.remove(pos);
        });
        AccountForAsset::<T, I>::remove(&asset_id);

        Ok(())
    }

    fn transfer(dest_account: &T::AccountId, asset_id: &AssetId<T>) -> dispatch::DispatchResult {
        let owner = Self::owner_of(&asset_id);
        ensure!(
            owner != T::AccountId::default(),
            Error::<T, I>::NonexistentAsset
        );

        ensure!(
            Self::total_for_account(dest_account) < T::UserAssetLimit::get(),
            Error::<T, I>::TooManyAssetsForAccount
        );

        let xfer_asset = IdentifiedAsset::<AssetId<T>, <T as Trait<I>>::AssetInfo> {
            id: *asset_id,
            asset: <T as Trait<I>>::AssetInfo::default(),
        };

        TotalForAccount::<T, I>::mutate(&owner, |total| *total -= 1);
        TotalForAccount::<T, I>::mutate(dest_account, |total| *total += 1);
        let asset = AssetsForAccount::<T, I>::mutate(owner, |assets| {
            let pos = assets
                .binary_search(&xfer_asset)
                .expect("We already checked that we have the correct owner; qed");
            assets.remove(pos)
        });
        AssetsForAccount::<T, I>::mutate(dest_account, |assets| {
            match assets.binary_search(&asset) {
                Ok(_pos) => {} // should never happen
                Err(pos) => assets.insert(pos, asset),
            }
        });
        AccountForAsset::<T, I>::insert(&asset_id, &dest_account);

        Ok(())
    }
}
