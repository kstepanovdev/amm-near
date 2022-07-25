#[bin/bash]

#export your ID
export ID=v0570k.testnet

#create token A and deploy the contract for it. Then init contract with default meta.
near create-account token_a.$ID --masterAccount=$ID --initialBalance=10;
NEAR_ENV=testnet near deploy --wasmFile res/fungible_token.wasm --accountId=token_a.$ID;
near call token_a.$ID new_default_meta '{"owner_id": "token_a.'$ID'","total_supply": "1000000"}' --accountId $ID;

#create token B and deploy the contract for it. Then init contract with default meta.
near create-account token_b.$ID --masterAccount=$ID --initialBalance=10;
NEAR_ENV=testnet near deploy --wasmFile res/fungible_token.wasm --accountId=token_b.$ID;
near call token_b.$ID new_default_meta '{"owner_id": "token_b.'$ID'","total_supply": "1000000"}' --accountId $ID;

#create Alice and Bob accounts
near create-account alice.$ID --masterAccount=$ID --initialBalance=10;
near create-account bob.$ID --masterAccount=$ID --initialBalance=10;

#create AMM account and deploy the contract for it
near create-account amm.$ID --masterAccount=$ID --initialBalance=10;
NEAR_ENV=testnet near deploy --wasmFile res/amm.wasm --accountId=amm.$ID;

#pay for account registration
near call token_a.$ID storage_deposit '{"account_id": "alice.'$ID'"}' --accountId $ID --deposit 1 --gas 300000000000000;
near call token_a.$ID storage_deposit '{"account_id": "'$ID'"}' --accountId $ID --deposit 1 --gas 300000000000000;
near call token_a.$ID storage_deposit '{"account_id": "bob.'$ID'"}' --accountId $ID --deposit 1 --gas 300000000000000;

near call token_b.$ID storage_deposit '{"account_id": "alice.'$ID'"}' --accountId $ID --deposit 1 --gas 300000000000000;
near call token_b.$ID storage_deposit '{"account_id": "'$ID'"}' --accountId $ID --deposit 1 --gas 300000000000000;
near call token_b.$ID storage_deposit '{"account_id": "bob.'$ID'"}' --accountId $ID --deposit 1 --gas 300000000000000;

#send some A tokens to Alice, Bob and AMM
near call token_a.$ID ft_transfer '{"receiver_id": "alice.'$ID'", "amount": "50000"}' --accountId token_a.$ID --depositYocto 1;
near call token_a.$ID ft_transfer '{"receiver_id": "bob.'$ID'", "amount": "1"}' --accountId token_a.$ID --depositYocto 1;
near call token_a.$ID ft_transfer '{"receiver_id": "'$ID'", "amount": "1000"}' --accountId token_a.$ID --depositYocto 1;

#send some B tokens to Alice, Bob and AMM
near call token_b.$ID ft_transfer '{"receiver_id": "alice.'$ID'", "amount": "1"}' --accountId token_b.$ID --depositYocto 1;
near call token_b.$ID ft_transfer '{"receiver_id": "bob.'$ID'", "amount": "10000"}' --accountId token_b.$ID --depositYocto 1;
near call token_b.$ID ft_transfer '{"receiver_id": "'$ID'", "amount": "10000"}' --accountId token_b.$ID --depositYocto 1;


# init AMM contract
near call amm.$ID new '{
    "owner_id": "amm.'$ID'",
    "a_contract": "token_a.'$ID'",
    "b_contract": "token_b.'$ID'"
}' --accountId amm.$ID --gas 100000000000000;
