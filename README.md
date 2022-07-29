# Automated Market Maker (AMM) built with NEAR Protocol
Prototype of AMM made with NEAR Protocol. It doesn't have any off-chain frontend.

## Getting started

### Contract building
To build a contract from the source you should execute build.sh script. It compiles source code to .wasm and copies it to the /res folder. It might be helpful if you decide to modify the source.

### Contract deployment
The simplest way to deploy the contract is using a deploy.sh script. It contains all required steps with commentary that can help you modify this script. You can almost freely play with different parameters (NB: be sure you've changed credentials in the "export" section for your own. If for some reason you don't want to use deploy.sh you can deploy a contract on your own. However you'll probably want to do the following steps:

1. Create fungible tokens A and B followed by NEP-141 standard. For their contracts, it's ok to use source code/wasm from [NEAR example](https://github.com/near-examples/FT) repo.
2. It's a good idea to create accounts for several users. Be careful because FT isn't the same as a native NEAR token and you must call FT's storage_deposit for each desired FT. Otherwise, you won't be able to send/accept money.
3. Don't forget to refill created wallets with some amount of tokens. It's not necessary, but helpful.
4. Call .new method on created AMM contract. The simplest way to do this is using NEAR CLI.

## Testing
Since `near-sdk-sim` is deprecated, integration tests are made with `workspaces-rs`. It uses `tokio.rs`, so tests are async. Right now test are a little bit overcomplicated and bloated, also they test only "happy path". They're located at [tests](https://github.com/kstepanovdev/amm-near/tree/master/tests). To run tests you probably want to use `sh test.sh`, but simple `cargo test` is possible (NB: if you changed the contract, be sure you rebuilt it). If you want to get something from `println!` macro inside your tests, use `cargo test -- --nocapture`.

**Get more info at:**

* [Rust Smart Contract Quick Start](https://docs.near.org/docs/develop/contracts/rust/intro)
* [Rust SDK Book](https://www.near-sdk.io/)
* [Near Nomicon book](https://nomicon.io/)
