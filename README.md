# Non-Fungible Token FRAME Pallet

This is a [FRAME](https://substrate.dev/docs/en/knowledgebase/runtime/frame) pallet that defines and implements a
[non-fungible token (NFT)](https://en.wikipedia.org/wiki/Non-fungible_token) interface.

## Tests

Refer to the [mock runtime](src/mock.rs) and [provided tests](src/tests.rs) to see the NFT implementation in action.

## Acknowledgements

This project was inspired by works such as the following:

- [The ERC-721 specification](https://eips.ethereum.org/EIPS/eip-721)
- [OpenZeppelin's ERC-721 implementation](https://github.com/OpenZeppelin/openzeppelin-contracts/tree/master/contracts/token/ERC721)
- [the original Substratekitties project](https://www.shawntabrizi.com/substrate-collectables-workshop/#/), by
  [@shawntabrizi](https://github.com/shawntabrizi/)
- [Substratekitties from SubstrateCourse](https://github.com/SubstrateCourse/substrate-kitties), by [@xlc](https://github.com/xlc/)

Thanks to the following people who helped me overcome my relatively limited understanding of Rust.

- [@JoshOrndoff](https://github.com/JoshOrndorff/)
- [@riusricardo](https://github.com/riusricardo/)
- [@rphmeier](https://github.com/rphmeier/)
- [@thiolliere](https://github.com/thiolliere/)
- [@gnunicorn](https://github.com/gnunicorn/)

## Upstream

This project was forked from
[the Substrate DevHub Pallet Template](https://github.com/substrate-developer-hub/substrate-pallet-template).
