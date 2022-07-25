#[bin/bash]

sh build.sh;
near delete amm.$ID $ID;
near create-account amm.$ID --masterAccount=$ID --initialBalance=10;
NEAR_ENV=testnet near deploy --wasmFile res/amm.wasm --accountId=amm.$ID;
