use std::fmt::Display;
use std::vec;

use itertools::Itertools;

use near_contract_standards::fungible_token::{metadata::FungibleTokenMetadata, FungibleToken};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, ext_contract, near_bindgen, AccountId, PanicOnDefault, PendingContractTx};

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct AMM {
    // NB: both token A and B adresses are in fact token's contract's addresses. Not the token themselves.
    pub owner_id: AccountId,
    pub tokens: UnorderedMap<AccountId, (FungibleToken, FungibleTokenMetadata, TickerInfo)>,
}

#[derive(Default, BorshSerialize, BorshDeserialize)]
pub struct TickerInfo {
    pub ratio_direction: TokenRate,
    pub percentage: f64,
    pub ratio: f64,
}
impl TickerInfo {
    fn update(&mut self, ratio: f64) {
        (self.ratio, self.ratio_direction, self.percentage) = match ratio.total_cmp(&self.ratio) {
            std::cmp::Ordering::Equal => (ratio, TokenRate::Unchanged, 0.0),
            std::cmp::Ordering::Less => (ratio, TokenRate::Decreased, self.ratio / ratio),
            std::cmp::Ordering::Greater => (ratio, TokenRate::Increased, ratio / self.ratio),
        }
    }
}
impl Display for TickerInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let direction_symbol = match self.ratio_direction {
            TokenRate::Unchanged => "=",
            TokenRate::Decreased => "v",
            TokenRate::Increased => "^",
        };
        write!(f, "({}, {})", direction_symbol, self.percentage)
    }
}

#[derive(Default, BorshSerialize, BorshDeserialize)]
pub enum TokenRate {
    #[default]
    Unchanged,
    Increased,
    Decreased,
}

// #[ext_contract(ext_self)]
// trait ExtSelf {
//     fn info(&self);
// }

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
        // let token_amm = create_token(&owner_id, b"amm".to_vec());

        let mut tokens = UnorderedMap::new(b"t");

        tokens.insert(&a_contract, &(a_token, a_meta, TickerInfo::default()));
        tokens.insert(&b_contract, &(b_token, b_meta, TickerInfo::default()));

        Self { owner_id, tokens }
    }

    pub fn info(&self) -> String {
        let mut res = "".to_string();
        for (token, info) in &self.tokens {
            res = format!(
                "{}Token {}. Ticker: {}, decimals: {};\n",
                res, token, info.2, info.1.decimals
            );
        }
        res
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

        let new_ratio = buy_token.0.total_supply as f64 / sell_token.0.total_supply as f64;
        sell_token.2.update(new_ratio);
        buy_token.2.update(new_ratio);
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

pub fn token_to_yocto(token_amount: &str) -> u128 {
    let mut vec_over_amount = token_amount.split('.').collect::<Vec<&str>>();
    if vec_over_amount.get(1).is_none() {
        vec_over_amount.push("0")
    };

    let (integer_part, decimal_part) = vec_over_amount
        .iter()
        .map(|x| x.parse::<u128>().unwrap_or(0))
        .collect_tuple()
        .unwrap();

    println!("{:?} {:?}", integer_part, decimal_part);

    {
        let int_part = integer_part * 10u128.pow(24);
        let decimal_part = {
            let amount_of_decimals = decimal_part.to_string().len() as u32;
            decimal_part * 10u128.pow(24 - amount_of_decimals)
        };
        int_part + decimal_part
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use near_sdk::test_utils::{accounts, get_logs, VMContextBuilder};
    use near_sdk::{testing_env, AccountId, Balance};

    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor.clone())
            .predecessor_account_id(predecessor);
        builder
    }

    #[test]
    fn test_conversion_whole_numbers() {
        assert_eq!(token_to_yocto("50"), 50 * 10u128.pow(24))
    }

    #[test]
    fn test_conversion_with_decimals() {
        let whole_part = 265 * 10u128.pow(24);
        let decimal_part = 555 * 10u128.pow(24 - 3);

        assert_eq!(token_to_yocto("265.555"), whole_part + decimal_part)
    }
}
