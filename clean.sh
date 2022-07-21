#[bin/bash]

#export your ID
export ID=v0570k.testnet

#delete all created account
near delete amm.$ID $ID;
near delete token_a.$ID $ID;
near delete token_b.$ID $ID;
near delete alice.$ID $ID;
near delete bob.$ID $ID;
