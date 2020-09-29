[![Compatible with Substrate v2.0.0](https://img.shields.io/badge/Substrate-v2.0.0-E6007A)](https://github.com/paritytech/substrate/releases/tag/v2.0.0)

# Commodities FRAME Pallet: NFTs for Substrate

This is a [FRAME](https://substrate.dev/docs/en/knowledgebase/runtime/frame) pallet that defines and implements an
interface for managing a set of [non-fungible tokens (NFTs)](https://en.wikipedia.org/wiki/Non-fungible_token). Assets
have an owner and can be created, destroyed and transferred.

## Interface

This package defines [a public trait](src/nft.rs) (Rust interface) for working with NFTs: the `UniqueAssets` trait.

## `UniqueAssets` Trait

This trait is generic with respect to a type that is used to identify asset owners - the `AccountId` type. Assets with
equivalent attributes (as defined by the `AssetInfo` type) **must** have equal `AssetId`s and assets with different
`AssetId`s **must not** have equivalent attributes.

### Types

- `AssetId`: a URI for an asset
- `AssetInfo`: a set of attributes that uniquely describes an asset
- `AssetLimit`: the maximum number of assets, expressed as an unsigned 128-bit integer, that may exist in this set at
  once
- `UserAssetLimit`: the maximum number of assets, expressed as an unsigned 64-bit integer, that any single account may
  own from this set at once

### Functions

- `total() -> u128`: returns the total number of assets in this set of assets
- `burned() -> u128`: returns the total number of assets from this set that have been burned
- `total_for_account(AccountId) -> u64`: returns the total number of asset from this set that are owned by a given
  account
- `assets_for_account(AccountId) -> Vec<(AssetId, AssetInfo)>`: returns the list of assets from this set that are owned
  by a given account
- `owner_of(AssetId) -> AccountId`: returns the ID of the account that owns the given asset from this set
- `mint(AccountId, AssetInfo) -> Result<AssetID, DispatchError>`: use the given attributes to create a new unique asset
  that belongs to this set and assign ownership of it to the given account
  - Failure cases: asset duplication, asset limit reached for set, asset limit for this set reached for account
- `burn(AssetId) -> DispatchResult`: destroy the given asset
  - Failure cases: asset doesn't exist
- `transfer(AccountId, AssetId) -> DispatchResult`: transfer ownership of the given asset from this set from its current
  owner to a given target account
  - Failure cases: asset doesn't exist, asset limit for this set reached for target account

## Reference Implementation

The [reference implementation](src/lib.rs) defined in this project is referred to as a "commodity" - a unique asset that
is designed for frequent trading. In order to optimize for this use case, _sorted_ lists of assets are stored per owner.
Although maintaining a sorted list is trivial with Rust vectors, which implement a binary search API that can be used
for sorted insertion, it introduces significant overhead when an asset is _created_ because the entire list must be
decoded from the backing trie in order to insert the new asset in the correct spot. Maintaining a sorted asset list is
desireable for the commodity use case, however, because it allows assets to be efficiently located when destroying or
transferring them. An alternative implementation, the Keepsake pallet, is in the works :rocket:

## Tests

Refer to the [mock runtime](src/mock.rs) and [provided tests](src/tests.rs) to see the NFT implementation in action.

## Test Project

In order to help develop this pallet, it is being consumed by
[a test project](https://github.com/danforbes/substratekitties) - a work-in-progress update to
[the original Substratekitties tutorial](https://github.com/shawntabrizi/substratekitties).

## Acknowledgements

This project was inspired by works such as the following:

- [The ERC-721 specification](https://eips.ethereum.org/EIPS/eip-721)
- [OpenZeppelin's ERC-721 implementation](https://github.com/OpenZeppelin/openzeppelin-contracts/tree/master/contracts/token/ERC721)
- [the original Substratekitties project](https://www.shawntabrizi.com/substrate-collectables-workshop/#/), by
  [@shawntabrizi](https://github.com/shawntabrizi/)
- [Substratekitties from SubstrateCourse](https://github.com/SubstrateCourse/substrate-kitties), by
  [@xlc](https://github.com/xlc/)

Thanks to the following people who helped me overcome my relatively limited understanding of Rust.

- [@JoshOrndoff](https://github.com/JoshOrndorff/)
- [@riusricardo](https://github.com/riusricardo/)
- [@rphmeier](https://github.com/rphmeier/)
- [@thiolliere](https://github.com/thiolliere/)
- [@gnunicorn](https://github.com/gnunicorn/)

## Upstream

This project was forked from
[the Substrate DevHub Pallet Template](https://github.com/substrate-developer-hub/substrate-pallet-template).
