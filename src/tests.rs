// Tests to be written here

use crate::mock::*;
use crate::nft::UniqueAssets;
use crate::*;
use frame_support::{assert_err, assert_ok, Hashable};
use sp_core::H256;

#[test]
fn mint() {
    new_test_ext().execute_with(|| {
        assert_eq!(SUT::total(), 0);
        assert_eq!(SUT::total_for_account(1), 0);
        assert_eq!(<SUT as UniqueAssets<_>>::total(), 0);
        assert_eq!(<SUT as UniqueAssets<_>>::total_for_account(&1), 0);
        assert_eq!(
            SUT::account_for_asset::<H256>(Vec::<u8>::default().blake2_256().into()),
            0
        );

        assert_ok!(SUT::mint(Origin::root(), 1, Vec::<u8>::default()));

        assert_eq!(SUT::total(), 1);
        assert_eq!(<SUT as UniqueAssets<_>>::total(), 1);
        assert_eq!(SUT::burned(), 0);
        assert_eq!(<SUT as UniqueAssets<_>>::burned(), 0);
        assert_eq!(SUT::total_for_account(1), 1);
        assert_eq!(<SUT as UniqueAssets<_>>::total_for_account(&1), 1);
        let assets_for_account = SUT::assets_for_account::<u64>(1);
        assert_eq!(assets_for_account.len(), 1);
        assert_eq!(
            assets_for_account[0].id,
            Vec::<u8>::default().blake2_256().into()
        );
        assert_eq!(assets_for_account[0].asset, Vec::<u8>::default());
        assert_eq!(
            SUT::account_for_asset::<H256>(Vec::<u8>::default().blake2_256().into()),
            1
        );
    });
}

#[test]
fn mint_err_non_admin() {
    new_test_ext().execute_with(|| {
        assert_err!(
            SUT::mint(Origin::signed(1), 1, Vec::<u8>::default()),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn mint_err_dupe() {
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, Vec::<u8>::default()));

        assert_err!(
            SUT::mint(Origin::root(), 2, Vec::<u8>::default()),
            Error::<Test, DefaultInstance>::AssetExists
        );
    });
}

#[test]
fn mint_err_max_user() {
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, vec![]));
        assert_ok!(SUT::mint(Origin::root(), 1, vec![0]));

        assert_err!(
            SUT::mint(Origin::root(), 1, vec![1]),
            Error::<Test, DefaultInstance>::TooManyAssetsForAccount
        );
    });
}

#[test]
fn mint_err_max() {
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, vec![]));
        assert_ok!(SUT::mint(Origin::root(), 2, vec![0]));
        assert_ok!(SUT::mint(Origin::root(), 3, vec![1]));
        assert_ok!(SUT::mint(Origin::root(), 4, vec![2]));
        assert_ok!(SUT::mint(Origin::root(), 5, vec![3]));

        assert_err!(
            SUT::mint(Origin::root(), 6, vec![4]),
            Error::<Test, DefaultInstance>::TooManyAssets
        );
    });
}

#[test]
fn burn() {
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, Vec::<u8>::default()));
        assert_ok!(SUT::burn(
            Origin::signed(1),
            Vec::<u8>::default().blake2_256().into()
        ));

        assert_eq!(SUT::total(), 0);
        assert_eq!(SUT::burned(), 1);
        assert_eq!(SUT::total_for_account(1), 0);
        assert_eq!(SUT::assets_for_account::<u64>(1), vec![]);
        assert_eq!(
            SUT::account_for_asset::<H256>(Vec::<u8>::default().blake2_256().into()),
            0
        );
    });
}

#[test]
fn burn_err_not_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, Vec::<u8>::default()));

        assert_err!(
            SUT::burn(Origin::signed(2), Vec::<u8>::default().blake2_256().into()),
            Error::<Test, DefaultInstance>::NotAssetOwner
        );
    });
}

#[test]
fn burn_err_not_exist() {
    new_test_ext().execute_with(|| {
        assert_err!(
            SUT::burn(Origin::signed(1), Vec::<u8>::default().blake2_256().into()),
            Error::<Test, DefaultInstance>::NotAssetOwner
        );
    });
}

#[test]
fn transfer() {
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, Vec::<u8>::default()));
        assert_ok!(SUT::transfer(
            Origin::signed(1),
            2,
            Vec::<u8>::default().blake2_256().into()
        ));

        assert_eq!(SUT::total(), 1);
        assert_eq!(SUT::burned(), 0);
        assert_eq!(SUT::total_for_account(1), 0);
        assert_eq!(SUT::total_for_account(2), 1);
        assert_eq!(SUT::assets_for_account::<u64>(1), vec![]);
        let assets_for_account = SUT::assets_for_account::<u64>(2);
        assert_eq!(assets_for_account.len(), 1);
        assert_eq!(
            assets_for_account[0].id,
            Vec::<u8>::default().blake2_256().into()
        );
        assert_eq!(assets_for_account[0].asset, Vec::<u8>::default());
        assert_eq!(
            SUT::account_for_asset::<H256>(Vec::<u8>::default().blake2_256().into()),
            2
        );
    });
}

#[test]
fn transfer_err_not_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, Vec::<u8>::default()));

        assert_err!(
            SUT::transfer(
                Origin::signed(0),
                2,
                Vec::<u8>::default().blake2_256().into()
            ),
            Error::<Test, DefaultInstance>::NotAssetOwner
        );
    });
}

#[test]
fn transfer_err_not_exist() {
    new_test_ext().execute_with(|| {
        assert_err!(
            SUT::transfer(
                Origin::signed(1),
                2,
                Vec::<u8>::default().blake2_256().into()
            ),
            Error::<Test, DefaultInstance>::NotAssetOwner
        );
    });
}

#[test]
fn transfer_err_max_user() {
    new_test_ext().execute_with(|| {
        assert_ok!(SUT::mint(Origin::root(), 1, vec![0]));
        assert_ok!(SUT::mint(Origin::root(), 1, vec![1]));
        assert_ok!(SUT::mint(Origin::root(), 2, Vec::<u8>::default()));
        assert_eq!(
            SUT::account_for_asset::<H256>(Vec::<u8>::default().blake2_256().into()),
            2
        );

        assert_err!(
            SUT::transfer(
                Origin::signed(2),
                1,
                Vec::<u8>::default().blake2_256().into()
            ),
            Error::<Test, DefaultInstance>::TooManyAssetsForAccount
        );
    });
}
