use std::default;
use std::fmt::{Debug, Display};

use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, Promise};

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct AMM {
    // NB: both token A and B adresses are in fact token's contract's addresses. Not the token themselves.
    pub owner_id: AccountId,
    pub tokens: UnorderedMap<AccountId, (FungibleToken, FungibleTokenMetadata, TickerInfo)>,
}

#[derive(Default, BorshSerialize, BorshDeserialize)]
pub struct TickerInfo {
    pub rate: TokenRate,
    pub percentage: u128,
}
impl Display for TickerInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Default, BorshSerialize, BorshDeserialize)]
pub enum TokenRate {
    #[default]
    Unchanged,
    Increase,
    Decrease,
}

// why do we need an onwer as a param? we could've just call
// either env::current_account_id() or env::predecessor_account_id();
#[near_bindgen]
impl AMM {
    #[init]
    pub fn new(
        owner_id: AccountId,
        a_contract: AccountId,
        a_meta: FungibleTokenMetadata,
        b_contract: AccountId,
        b_meta: FungibleTokenMetadata,
    ) -> Self {
        // create wallets for new tokens
        let a_token = create_token(&owner_id, b"a".to_vec());
        let b_token = create_token(&owner_id, b"b".to_vec());
        // let token_amm = create_token(&owner, b"amm".to_vec());

        let mut tokens = UnorderedMap::new(b"t");

        tokens.insert(&a_contract, &(a_token, a_meta, TickerInfo::default()));
        tokens.insert(&b_contract, &(b_token, b_meta, TickerInfo::default()));

        Self { owner_id, tokens }
    }

    pub fn info(&self) {
        for (token, info) in &self.tokens {
            println!(
                "Token {}. Ticker: {}, decimals: {};",
                token, info.2, info.1.decimals
            );
        }
    }

    pub fn swap(&mut self, buy_token_addr: AccountId, sell_token_addr: AccountId, amount: u128) {
        let mut buy_token = self
            .tokens
            .get(&buy_token_addr)
            .expect("Token not supported");
        let mut sell_token = self
            .tokens
            .get(&sell_token_addr)
            .expect("Token not supported");
        let pool_owner_id = &self.owner_id;
        let user_account_id = env::predecessor_account_id();

        let x = sell_token.0.internal_unwrap_balance_of(pool_owner_id);
        let y = buy_token.0.internal_unwrap_balance_of(pool_owner_id);
        let k = x * y;

        sell_token
            .0
            .internal_transfer(&user_account_id, pool_owner_id, amount, None);

        // operations with a floating pointer won't be implemented due to my lack of experience of such operations
        // y - (k / (x + dx))
        let buy_amount = y - (k / (x + amount));

        buy_token
            .0
            .internal_transfer(pool_owner_id, &user_account_id, buy_amount, None);

        self.tokens.insert(&buy_token_addr, &buy_token);
        self.tokens.insert(&sell_token_addr, &sell_token);
    }

    pub fn deposit(
        &mut self,
        token_a_addr: AccountId,
        token_b_addr: AccountId,
        amount_a: u128,
        amount_b: u128,
    ) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Only the owner may call this method"
        );

        let payer = env::predecessor_account_id();

        let mut token_a = self.tokens.get(&token_a_addr).expect("Token not supported");
        let mut token_b = self.tokens.get(&token_b_addr).expect("Token not supported");
        token_a
            .0
            .internal_transfer(&payer, &self.owner_id, amount_a, None);
        token_b
            .0
            .internal_transfer(&payer, &self.owner_id, amount_b, None);

        self.tokens.insert(&token_a_addr, &token_a);
        self.tokens.insert(&token_b_addr, &token_b);
    }
}

pub fn create_token(account_id: &AccountId, prefix: Vec<u8>) -> FungibleToken {
    let mut token = FungibleToken::new(prefix);
    token.internal_register_account(account_id);
    token
}

/*
 * the rest of this file sets up unit tests
 * to run these, the command will be:
 * cargo test --package rust-template -- --nocapture
 * Note: 'rust-template' comes from Cargo.toml's 'name' key
 */

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{get_logs, VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

    // part of writing unit tests is setting up a mock context
    // provide a `predecessor` here, it'll modify the default context
    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    // TESTS HERE
}
