// use amm::AMM;
// use fungible_token::ContractContract as Contract;
// use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FT_METADATA_SPEC};
// use near_sdk::{serde_json::json, json_types::U128};
// use near_sdk_sim::{deploy, init_simulator, to_yocto, ContractAccount, UserAccount};

// // Load in contract bytes at runtime
// near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
//     FT_WASM_BYTES => "res/fungible_token.wasm",
//     AMM_WASM_BYTES => "res/amm.wasm",
// }

// pub const TOKEN_A_ID: &str = "token_a";
// pub const TOKEN_B_ID: &str = "token_b";
// pub const AMM_ID: &str = "amm";

// pub fn add_account(contract_id: &str, user: &near_sdk_sim::UserAccount) {
//     user.call(
//         contract_id.parse().unwrap(),
//         "storage_deposit",
//         &json!({
//             "account_id": user.account_id()
//         })
//         .to_string()
//         .into_bytes(),
//         near_sdk_sim::DEFAULT_GAS / 2,
//         near_sdk::env::storage_byte_cost() * 125, // attached deposit
//     )
//     .assert_success();
// }

// pub fn init() -> (
//     UserAccount,
//     ContractAccount<Contract>,
//     ContractAccount<Contract>,
//     ContractAccount<AMM>,
//     UserAccount,
//     UserAccount
// ) {
//     let root = init_simulator(None);
//     let init_balance = U128(10_000_u128);

//     // create token A with its meta
//     let meta_a = FungibleTokenMetadata {
//         spec: FT_METADATA_SPEC.to_string(),
//         name: "A".to_string(),
//         symbol: "A".to_string(),
//         icon: None,
//         reference: None,
//         reference_hash: None,
//         decimals: 3,
//     };
//     let a_contract = deploy!(
//         contract: Contract,
//         contract_id: TOKEN_A_ID,
//         bytes: &FT_WASM_BYTES,
//         signer_account: root,
//         init_method: new(
//             root.account_id(),
//             init_balance,
//             meta_a
//         )
//     );

//     // create token B with its meta
//     let meta_b = FungibleTokenMetadata {
//         spec: FT_METADATA_SPEC.to_string(),
//         name: "B".to_string(),
//         symbol: "B".to_string(),
//         icon: None,
//         reference: None,
//         reference_hash: None,
//         decimals: 5,
//     };
//     let b_contract = deploy!(
//         contract: Contract,
//         contract_id: TOKEN_B_ID,
//         bytes: &FT_WASM_BYTES,
//         signer_account: root,
//         init_method: new(
//             root.account_id(),
//             init_balance.into(),
//             meta_b
//         )
//     );

//     // create AMM contract
//     let amm_contract = deploy!(
//         contract: AMM,
//         contract_id: AMM_ID,
//         bytes: &AMM_WASM_BYTES,
//         signer_account: root,
//         init_method: new(
//             root.account_id(),
//             a_contract.account_id(),
//             b_contract.account_id(),
//             meta_a,
//             meta_b
//         )
//     );

//     // create two users and wallets for them
//     let alice = root.create_user("alice".parse().unwrap(), to_yocto("100"));
//     add_account(TOKEN_A_ID, &alice);
//     add_account(TOKEN_B_ID, &alice);

//     let bob = root.create_user("bob".parse().unwrap(), to_yocto("100"));
//     add_account(TOKEN_A_ID, &bob);
//     add_account(TOKEN_B_ID, &bob);

//     (
//         root,
//         a_contract,
//         b_contract,
//         amm_contract,
//         alice,
//         bob,
//     )
// }
