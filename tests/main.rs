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
        .dev_deploy(&include_bytes!("../res/fungible_token.wasm").to_vec())
        .await?;
    let res = a_contract
        .call(worker, "new_default_meta")
        .args_json((a_contract.id(), "50000"))?
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
        .args_json((b_contract.id(), "20000"))?
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

    Ok((owner, a_contract, b_contract, alice, bob, amm_contract))
}

#[tokio::test]
async fn init_amm() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (owner, a_contract, b_contract, _, _, amm_contract) = init(&worker).await?;

    let res = amm_contract
        .call(&worker, "new")
        .args_json((owner.id(), a_contract.id(), b_contract.id()))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(res.is_success());

    Ok(())
}
