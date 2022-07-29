use near_sdk::json_types::U128;
use near_units::{parse_gas, parse_near};
use serde_json::json;
use workspaces::result::CallExecutionDetails;
use workspaces::{network::Sandbox, Account, Contract, Worker};
use workspaces::{prelude::*, sandbox, DevNetwork};

async fn init(
    worker: &Worker<Sandbox>,
) -> anyhow::Result<(Account, Contract, Contract, Account, Account, Contract)> {
    let owner = worker.root_account()?;

    // Create the FT contract for A and an account for it.
    let a_contract = worker
        .dev_deploy(include_bytes!("../res/fungible_token.wasm").as_ref())
        .await?;
    let res = a_contract
        .call(worker, "new_default_meta")
        .args_json((a_contract.id(), "5000000"))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success());

    // Create the FT contract for B and an account for it.
    let b_contract = worker
        .dev_deploy(include_bytes!("../res/fungible_token.wasm").as_ref())
        .await?;
    let res = b_contract
        .call(worker, "new_default_meta")
        .args_json((b_contract.id(), "2000000"))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success());

    // Create AMM contract and an account for it.
    let amm_contract = worker
        .dev_deploy(include_bytes!("../res/amm.wasm").as_ref())
        .await?;

    // Create Alice and Bob
    let alice = owner
        .create_subaccount(worker, "alice")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .into_result()?;

    let bob = owner
        .create_subaccount(worker, "bob")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .into_result()?;

    // token "A" storage deposit for Alice and Bob
    let res = owner
        .call(worker, a_contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": bob.id(),
        }))?
        .deposit(parse_near!("1 N"))
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success());

    let res = owner
        .call(worker, a_contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": alice.id(),
        }))?
        .deposit(parse_near!("1 N"))
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success());

    // token "B" storage deposit for Alice and Bob
    let res = owner
        .call(worker, b_contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": bob.id(),
        }))?
        .deposit(parse_near!("1 N"))
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success());
    let res = owner
        .call(worker, b_contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": alice.id(),
        }))?
        .deposit(parse_near!("1 N"))
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success());

    // send some "A" tokens for Alice and Bob
    let res = a_contract
        .call(worker, "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": alice.id(),
            "amount": U128(500_000)
        }))?
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success());
    let res = a_contract
        .call(worker, "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": bob.id(),
            "amount": U128(100_000)
        }))?
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success());

    // send some "B" tokens for Alice and Bob
    let res = b_contract
        .call(worker, "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": alice.id(),
            "amount": U128(20_000)
        }))?
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success());
    let res = b_contract
        .call(worker, "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": bob.id(),
            "amount": U128(700_000)
        }))?
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success());

    // deposit for root account, send tokens for it
    let res = alice
        .call(worker, a_contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": owner.id(),
        }))?
        .deposit(parse_near!("1 N"))
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success());

    let res = alice
        .call(worker, b_contract.id(), "storage_deposit")
        .args_json(serde_json::json!({
            "account_id": owner.id(),
        }))?
        .deposit(parse_near!("1 N"))
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success());

    let res = a_contract
        .call(worker, "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": owner.id(),
            "amount": U128(20_000)
        }))?
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success());
    let res = b_contract
        .call(worker, "ft_transfer")
        .args_json(serde_json::json!({
            "receiver_id": owner.id(),
            "amount": U128(20_000)
        }))?
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success());

    Ok((owner, a_contract, b_contract, alice, bob, amm_contract))
}

// #[tokio::test]
// async fn init_amm() -> anyhow::Result<()> {
//     let worker = workspaces::sandbox().await?;
//     let (
//         owner,
//         a_contract,
//         b_contract,
//         _alice,
//         _bob,
//         amm_contract,
//         _amm
//     ) = init(&worker).await?;

//     let res = amm_contract.call(&worker, "new")
//         .args_json(serde_json::json!({
//             "owner_id": owner.id(),
//             "a_contract": a_contract.id(),
//             "b_contract": b_contract.id(),
//         }))?
//         .gas(300_000_000_000_000)
//         .transact()
//         .await?;
//     assert!(res.is_success());

//     Ok(())
// }

// #[tokio::test]
// async fn get_info() -> anyhow::Result<()> {
//     let worker = workspaces::sandbox().await?;
//     let (owner, a_contract, b_contract, _, _, amm_contract) = init(&worker).await?;

//     let res = amm_contract
//         .call(&worker, "new")
//         .args_json((owner.id(), a_contract.id(), b_contract.id()))?
//         .gas(300_000_000_000_000)
//         .transact()
//         .await?;
//     assert!(res.is_success());

//     let res: String = amm_contract
//         .call(&worker, "info")
//         .gas(300_000_000_000_000)
//         .transact()
//         .await?
//         .json()?;
//     let er = format!(
//         "Token address: {}. Token name: {}. Decimals: {}. Ticker: TBD. Balance: {}; Token address: {}. Token name: {}. Decimals: {}. Ticker: TBD. Balance: {}; Tokens ratio: 0",
//         a_contract.id(), "Example NEAR fungible token", 24, 50000,
//         b_contract.id(), "Example NEAR fungible token", 24, 20000,
//     );

//     assert_eq!(res, er);
//     Ok(())
// }

#[tokio::test]
async fn deposit() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (owner, a_contract, b_contract, alice, bob, amm_contract) = init(&worker).await?;

    let res = owner
        .call(&worker, amm_contract.id(), "new")
        .args_json(serde_json::json!({
            "owner_id": owner.id(),
            "a_contract": a_contract.id(),
            "b_contract": b_contract.id(),
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success());

    // deposit AMM account with 10_000 "A" coins. Later check it for consistency.
    let res = owner
        .call(&worker, a_contract.id(), "ft_transfer_call")
        .args_json(serde_json::json!({
            "receiver_id": amm_contract.id(),
            "amount": U128(10000),
            "msg": b_contract.id()
        }))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success());

    let res: U128 = bob
        .call(&worker, a_contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": amm_contract.id(),
        }))?
        .view()
        .await?
        .json()?;
    assert_eq!(res, U128(10000));

    // deposit AMM account with 5_000 "B" coins. Later check it for consistency.
    let res = owner
        .call(&worker, b_contract.id(), "ft_transfer_call")
        .args_json(serde_json::json!({
            "receiver_id": amm_contract.id(),
            "amount": U128(5000),
            "msg": a_contract.id()
        }))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await?;
    assert!(res.is_success());

    let res: U128 = bob
        .call(&worker, b_contract.id(), "ft_balance_of")
        .args_json(serde_json::json!({
            "account_id": amm_contract.id(),
        }))?
        .view()
        .await?
        .json()?;
    assert_eq!(res, U128(5000));

    let res: String = bob
        .call(&worker, amm_contract.id(), "info")
        .view()
        .await?
        .json()?;
    println!("{}", res);

    Ok(())
}
