//! # Unique Assets Interface
//!
//! This trait describes an abstraction over a set of unique assets, also known as non-fungible
//! tokens (NFTs).
//!
//! ## Overview
//!
//! Unique assets have an owner, identified by an account ID, and are defined by a common set of
//! attributes (the asset info type). An asset ID type distinguishes unique assets from one another.
//! Assets may be created (minted), destroyed (burned) or transferred.
//!
//! This abstraction is implemented by [pallet_commodities::Module](../struct.Module.html).

use frame_support::{
    dispatch::{result::Result, DispatchError, DispatchResult},
    traits::Get,
};
use sp_std::vec::Vec;

/// An interface over a set of unique assets.
/// Assets with equivalent attributes (as defined by the AssetInfo type) **must** have an equal ID
/// and assets with different IDs **must not** have equivalent attributes.
pub trait UniqueAssets<AccountId> {
    /// The type used to identify unique assets.
    type AssetId;
    /// The attributes that distinguish unique assets.
    type AssetInfo;
    /// The maximum number of this type of asset that may exist (minted - burned).
    type AssetLimit: Get<u128>;
    /// The maximum number of this type of asset that any single account may own.
    type UserAssetLimit: Get<u64>;

    /// The total number of this type of asset that exists (minted - burned).
    fn total() -> u128;
    /// The total number of this type of asset that has been burned (may overflow).
    fn burned() -> u128;
    /// The total number of this type of asset owned by an account.
    fn total_for_account(account: &AccountId) -> u64;
    /// The set of unique assets owned by an account.
    fn assets_for_account(account: &AccountId) -> Vec<(Self::AssetId, Self::AssetInfo)>;
    /// The ID of the account that owns an asset.
    fn owner_of(asset_id: &Self::AssetId) -> AccountId;

    /// Use the provided asset info to create a new unique asset for the specified user.
    /// This method **must** return an error in the following cases:
    /// - The asset, as identified by the asset info, already exists.
    /// - The specified owner account has already reached the user asset limit.
    /// - The total asset limit has already been reached.
    fn mint(
        owner_account: &AccountId,
        asset_info: Self::AssetInfo,
    ) -> Result<Self::AssetId, DispatchError>;
    /// Destroy an asset.
    /// This method **must** return an error in the following case:
    /// - The asset with the specified ID does not exist.
    fn burn(asset_id: &Self::AssetId) -> DispatchResult;
    /// Transfer ownership of an asset to another account.
    /// This method **must** return an error in the following cases:
    /// - The asset with the specified ID does not exist.
    /// - The destination account has already reached the user asset limit.
    fn transfer(dest_account: &AccountId, asset_id: &Self::AssetId) -> DispatchResult;
}
