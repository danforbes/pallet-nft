//! # Unique Assets Implementation: Commodities
//!
//! This pallet exposes capabilities for managing unique assets, also known as
//! non-fungible tokens (NFTs).
//!
//! - [`pallet_commodities::Trait`](./trait.Trait.html)
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
//! trait in a way that is optimized for assets that are expected to be traded
//! frequently.
//!
//! ### Dispatchable Functions
//!
//! * [`mint`](./enum.Call.html#variant.mint) - Use the provided commodity info
//!   to create a new commodity for the specified user. May only be called by
//!   the commodity admin.
//!
//! * [`burn`](./enum.Call.html#variant.burn) - Destroy a commodity. May only be
//!   called by commodity owner.
//!
//! * [`transfer`](./enum.Call.html#variant.transfer) - Transfer ownership of
//!   a commodity to another account. May only be called by current commodity
//!   owner.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::FullCodec;
use frame_support::{dispatch::DispatchResultWithPostInfo, ensure, traits::Get, Hashable};
use sp_runtime::{traits::Hash, DispatchError};
use sp_std::{fmt::Debug, vec::Vec};

pub mod nft;
pub use crate::nft::UniqueAssets;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;

/// The runtime system's hashing algorithm is used to uniquely identify commodities.
pub type CommodityId<T> = <T as frame_system::Config>::Hash;

/// Associates a commodity with its ID.
pub type Commodity<T> = (CommodityId<T>, <T as Config>::CommodityInfo);

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    #[pallet::generate_store(trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// The dispatch origin that is able to mint new instances of this type of commodity.
        type CommodityAdmin: EnsureOrigin<Self::Origin>;
        /// The data type that is used to describe this type of commodity.
        type CommodityInfo: Hashable + Parameter + Member + Debug + Default + FullCodec + Ord + MaybeSerializeDeserialize;
        /// The maximum number of this type of commodity that may exist (minted - burned).
        type CommodityLimit: Get<u128>;
        /// The maximum number of this type of commodity that any single account may own.
        type UserCommodityLimit: Get<u64>;
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new commodity from the provided commodity info and identify the specified
        /// account as its owner. The ID of the new commodity will be equal to the hash of the info
        /// that defines it, as calculated by the runtime system's hashing algorithm.
        ///
        /// The dispatch origin for this call must be the commodity admin.
        ///
        /// This function will throw an error if it is called with commodity info that describes
        /// an existing (duplicate) commodity, if the maximum number of this type of commodity already
        /// exists or if the specified owner already owns the maximum number of this type of
        /// commodity.
        ///
        /// - `owner_account`: Receiver of the commodity.
        /// - `commodity_info`: The information that defines the commodity.
        #[pallet::weight(10_000)]
        pub fn mint(
            origin: OriginFor<T>,
            owner_account: T::AccountId,
            commodity_info: T::CommodityInfo,
        ) -> DispatchResultWithPostInfo {
            T::CommodityAdmin::ensure_origin(origin)?;

            let commodity_id = <Self as UniqueAssets<_>>::mint(&owner_account, commodity_info)?;
            Self::deposit_event(Event::Minted(commodity_id, owner_account.clone()));
            Ok(().into())
        }

        /// Destroy the specified commodity.
        ///
        /// The dispatch origin for this call must be the commodity owner.
        ///
        /// - `commodity_id`: The hash (calculated by the runtime system's hashing algorithm)
        ///   of the info that defines the commodity to destroy.
        #[pallet::weight(10_000)]
        pub fn burn(
            origin: OriginFor<T>,
            commodity_id: CommodityId<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                who == Self::account_for_commodity(&commodity_id),
                Error::<T>::NotCommodityOwner
            );

            <Self as UniqueAssets<_>>::burn(&commodity_id)?;
            Self::deposit_event(Event::Burned(commodity_id.clone()));
            Ok(().into())
        }

        /// Transfer a commodity to a new owner.
        ///
        /// The dispatch origin for this call must be the commodity owner.
        ///
        /// This function will throw an error if the new owner already owns the maximum
        /// number of this type of commodity.
        ///
        /// - `dest_account`: Receiver of the commodity.
        /// - `commodity_id`: The hash (calculated by the runtime system's hashing algorithm)
        ///   of the info that defines the commodity to destroy.
        #[pallet::weight(10_000)]
        pub fn transfer(
            origin: OriginFor<T>,
            dest_account: T::AccountId,
            commodity_id: CommodityId<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(
                who == Self::account_for_commodity(&commodity_id),
                Error::<T>::NotCommodityOwner
            );

            <Self as UniqueAssets<_>>::transfer(&dest_account, &commodity_id)?;
            Self::deposit_event(Event::Transferred(
                commodity_id.clone(),
                dest_account.clone(),
            ));
            Ok(().into())
        }
    }

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId", CommodityId<T> = "CommodityId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The commodity has been burned.
        Burned(CommodityId<T>),
        /// The commodity has been minted and distributed to the account.
        Minted(CommodityId<T>, T::AccountId),
        /// Ownership of the commodity has been transferred to the account.
        Transferred(CommodityId<T>, T::AccountId),
    }

    /// Error for the nicks module.
    #[pallet::error]
    pub enum Error<T> {
        // Thrown when there is an attempt to mint a duplicate commodity.
        CommodityExists,
        // Thrown when there is an attempt to burn or transfer a nonexistent commodity.
        NonexistentCommodity,
        // Thrown when someone who is not the owner of a commodity attempts to transfer or burn it.
        NotCommodityOwner,
        // Thrown when the commodity admin attempts to mint a commodity and the maximum number of this
        // type of commodity already exists.
        TooManyCommodities,
        // Thrown when an attempt is made to mint or transfer a commodity to an account that already
        // owns the maximum number of this type of commodity.
        TooManyCommoditiesForAccount,
    }

    #[pallet::type_value]
    pub(super) fn DefaultForTotal() -> u128 {
        0u128
    }

    #[pallet::type_value]
    pub(super) fn DefaultForBurned() -> u128 {
        0u128
    }

    /// The total number of this type of commodity that exists (minted - burned).
    #[pallet::storage]
    #[pallet::getter(fn total)]
    pub(super) type Total<T: Config> = StorageValue<_, u128, ValueQuery, DefaultForTotal>;

    #[pallet::storage]
    #[pallet::getter(fn burned)]
    pub(super) type Burned<T: Config> = StorageValue<_, u128, ValueQuery, DefaultForBurned>;

    /// The total number of this type of commodity owned by an account.
    #[pallet::storage]
    #[pallet::getter(fn total_for_account)]
    pub(super) type TotalForAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u64, ValueQuery>;

    /// A mapping from an account to a list of all of the commodities of this type that are owned by it.
    #[pallet::storage]
    #[pallet::getter(fn commodities_for_account)]
    pub(super) type CommoditiesForAccount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, Vec<CommodityId<T>>, ValueQuery>;

    /// A mapping from a commodity ID to the account that owns it.
    #[pallet::storage]
    #[pallet::getter(fn account_for_commodity)]
    pub(super) type AccountForCommodity<T: Config> =
        StorageMap<_, Identity, CommodityId<T>, T::AccountId, ValueQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub balances: Vec<(T::AccountId, Vec<T::CommodityInfo>)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                balances: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            for (who, assets) in self.balances.iter() {
                for asset in assets {
                    match <Module<T> as UniqueAssets<T::AccountId>>::mint(who, asset.clone()) {
                        Ok(_) => {}
                        Err(err) => {
                            panic!("{:?}", err)
                        }
                    }
                }
            }
        }
    }
}

impl<T: Config> UniqueAssets<T::AccountId> for Module<T> {
    type AssetId = CommodityId<T>;
    type AssetInfo = T::CommodityInfo;
    type AssetLimit = T::CommodityLimit;
    type UserAssetLimit = T::UserCommodityLimit;

    fn total() -> u128 {
        Self::total()
    }

    fn burned() -> u128 {
        Self::burned()
    }

    fn total_for_account(account: &T::AccountId) -> u64 {
        Self::total_for_account(account)
    }

    fn assets_for_account(account: &T::AccountId) -> Vec<CommodityId<T>> {
        Self::commodities_for_account(account)
    }

    fn owner_of(commodity_id: &CommodityId<T>) -> T::AccountId {
        Self::account_for_commodity(commodity_id)
    }

    fn mint(
        owner_account: &T::AccountId,
        commodity_info: <T as Config>::CommodityInfo,
    ) -> Result<CommodityId<T>, DispatchError> {
        let commodity_id = T::Hashing::hash_of(&commodity_info);

        ensure!(
            !AccountForCommodity::<T>::contains_key(&commodity_id),
            Error::<T>::CommodityExists
        );

        ensure!(
            Self::total_for_account(owner_account) < T::UserCommodityLimit::get(),
            Error::<T>::TooManyCommoditiesForAccount
        );

        ensure!(
            Self::total() < T::CommodityLimit::get(),
            Error::<T>::TooManyCommodities
        );

        Total::<T>::mutate(|total| *total += 1);
        TotalForAccount::<T>::mutate(owner_account, |total| *total += 1);
        CommoditiesForAccount::<T>::mutate(owner_account, |commodities| {
            match commodities.binary_search(&commodity_id) {
                Ok(_pos) => {} // should never happen
                Err(pos) => commodities.insert(pos, commodity_id),
            }
        });
        AccountForCommodity::<T>::insert(commodity_id, &owner_account);

        Ok(commodity_id)
    }

    fn burn(commodity_id: &CommodityId<T>) -> DispatchResultWithPostInfo {
        let owner = Self::owner_of(commodity_id);
        ensure!(
            owner != T::AccountId::default(),
            Error::<T>::NonexistentCommodity
        );

        let (burn_commodity, _) = (*commodity_id, <T as Config>::CommodityInfo::default());

        Total::<T>::mutate(|total| *total -= 1);
        Burned::<T>::mutate(|total| *total += 1);
        TotalForAccount::<T>::mutate(&owner, |total| *total -= 1);
        CommoditiesForAccount::<T>::mutate(owner, |commodities| {
            let pos = commodities
                .binary_search(&burn_commodity)
                .expect("We already checked that we have the correct owner; qed");
            commodities.remove(pos);
        });
        AccountForCommodity::<T>::remove(&commodity_id);

        Ok(().into())
    }

    fn transfer(
        dest_account: &T::AccountId,
        commodity_id: &CommodityId<T>,
    ) -> DispatchResultWithPostInfo {
        let owner = Self::owner_of(&commodity_id);
        ensure!(
            owner != T::AccountId::default(),
            Error::<T>::NonexistentCommodity
        );

        ensure!(
            Self::total_for_account(dest_account) < T::UserCommodityLimit::get(),
            Error::<T>::TooManyCommoditiesForAccount
        );

        let (xfer_commodity, _) = (*commodity_id, <T as Config>::CommodityInfo::default());

        TotalForAccount::<T>::mutate(&owner, |total| *total -= 1);
        TotalForAccount::<T>::mutate(dest_account, |total| *total += 1);
        let commodity = CommoditiesForAccount::<T>::mutate(owner, |commodities| {
            let pos = commodities
                .binary_search(&xfer_commodity)
                .expect("We already checked that we have the correct owner; qed");
            commodities.remove(pos)
        });
        CommoditiesForAccount::<T>::mutate(dest_account, |commodities| {
            match commodities.binary_search(&commodity) {
                Ok(_pos) => {} // should never happen
                Err(pos) => {
                    commodities.insert(pos, commodity);
                }
            }
        });
        AccountForCommodity::<T>::insert(&commodity_id, &dest_account);

        Ok(().into())
    }
}
