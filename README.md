[![Compatible with Substrate v2.0.0-rc6](https://img.shields.io/badge/Substrate-v2.0.0--rc6-E6007A)](https://github.com/paritytech/substrate/releases/tag/v2.0.0-rc6)

# Commodities FRAME Pallet: NFTs for Substrate

This is a [FRAME](https://substrate.dev/docs/en/knowledgebase/runtime/frame) pallet that defines and implements a
[non-fungible token (NFT)](https://en.wikipedia.org/wiki/Non-fungible_token) interface as well as an interface for
managing a set of such assets, including asset ownership, creation, destruction and transfer.

## Interface

This package defines [two public traits](src/nft.rs) (Rust interfaces) for working with NFTs: the `NFT` trait and the
`UniqueAssets` trait.

## `NFT` Trait

The `NFT` trait uses two types to define a unique asset:

- `ID`: a URI for the asset
- `Info`: a set of attributes that uniquely describe the asset

Assets with equivalent attributes (as defined by the `Info` type) **must** have an equal `ID` and assets with different
`ID`s **must not** have equivalent attributes.

## `UniqueAssets` Trait

This trait is generic with respect to a type that implements the `NFT` trait; it defines the type abstractions and
public functions needed to manage a set of unique assets.

### Types

- `AccountId`: the type used to identify asset owners
- `AssetLimit`: the maximum number of assets, expressed as an unsigned 128-bit integer, that may exist in this set at
  once
- `UserAssetLimit`: the maximum number of assets, expressed as an unsigned 64-bit integer, that any single account may
  own from this set at once

### Functions

- `total() -> u128`: returns the total number of assets in this set of assets
- `burned() -> u128`: returns the total number of assets from this set that have been burned
- `total_for_account(AccountId) -> u64`: returns the total number of asset from this set that are owned by a given
  account
- `assets_for_account(AccountId) -> Vec<NFT>`: returns the list of assets from this set that are owned by a given
  account
- `owner_of(NFT::Id) -> AccountId`: returns the ID of the account that owns the given asset from this set
- `mint(AccountId, NFT::Info) -> Result<AssetID, DispatchError>`: use the given attributes to create a new unique asset
  that belongs to this set and assign ownership of it to the given account
  - Failure cases: asset duplication, asset limit reached for set, asset limit for this set reached for account
- `burn(NFT::Id) -> DispatchResult`: destroy the given asset
  - Failure cases: asset doesn't exist
- `transfer(AccountId, NFT::Id) -> DispatchResult`: transfer ownership of the given asset from this set from its current
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
