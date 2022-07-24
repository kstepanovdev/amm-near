use std::thread::panicking;
use std::vec;

use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_contract_standards::non_fungible_token::Token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::{env, ext_contract, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, Promise};

pub const GAS: Gas = Gas(5_000_000_000_000);
const MIN_STORAGE: Balance = 1_000_000_000_000_000_000_000;

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct AMM {
    pub owner_id: AccountId,
    pub tokens: UnorderedMap<AccountId, TokenInfo>,
    pub tokens_ratio: f64,
}

#[derive(Default, BorshSerialize, BorshDeserialize)]
pub struct TokenInfo {
    name: String,
    decimals: u8,
    balance: u128,
}

// #[derive(Default, BorshSerialize, BorshDeserialize)]
// pub struct TickerInfo {
//     pub ratio_direction: TokenRate,
//     pub percentage: f64,
//     pub ratio: f64,
// }
// impl TickerInfo {
//     fn update(&mut self, ratio: f64) {
//         (self.ratio, self.ratio_direction, self.percentage) = match ratio.total_cmp(&self.ratio) {
//             std::cmp::Ordering::Equal => (ratio, TokenRate::Unchanged, 0.0),
//             std::cmp::Ordering::Less => (&ratio, TokenRate::Decreased, self.ratio / ratio),
//             std::cmp::Ordering::Greater => (ratio, Tok&enRate::Increased, ratio / self.ratio),
//         }
//     }
// }
// impl Display for TickerInfo {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let direction_symbol = match self.ratio_direction {
//             TokenRate::Unchanged => "=",
//             TokenRate::Decreased => "v",
//             TokenRate::Increased => "^",
//         };
//         write!(f, "({}, {})", direction_symbol, self.percentage)
//     }
// }

#[derive(Default, BorshSerialize, BorshDeserialize)]
pub enum TokenRate {
    #[default]
    Unchanged,
    Increased,
    Decreased,
}

#[ext_contract(ext_ft)]
trait Contract {
    fn ft_metadata(&self) -> Promise;
    fn ft_balance_of(&self, account_id: AccountId) -> Promise;
    fn ft_transfer(&self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn storage_deposit(&self, account_id: AccountId, registration_only: bool);
}

#[near_bindgen]
impl AMM {
    #[private]
    pub fn ft_balance_of_callback(
        &mut self,
        account_id: &AccountId,
        #[callback_unwrap] balance: U128,
    ) {
        let mut token_info = self.tokens.get(account_id).unwrap();
        token_info.balance = u128::from(balance);
        self.tokens.insert(account_id, &token_info);
    }

    #[private]
    pub fn ft_metadata_callback(
        &mut self,
        account_id: &AccountId,
        #[callback_unwrap] meta: FungibleTokenMetadata,
    ) {
        let token_info = TokenInfo {
            name: meta.name,
            decimals: meta.decimals,
            balance: 0_u128,
        };
        self.tokens.insert(account_id, &token_info);
    }

    #[private]
    pub fn ft_deposit_callback(&mut self) {}
}

#[near_bindgen]
impl AMM {
    #[init]
    pub fn new(owner_id: AccountId, a_contract: AccountId, b_contract: AccountId) -> Self {
        // create AMM account for owner to store the pool, then create wallets A and B for that account
        ext_ft::ext(a_contract.clone())
            .storage_deposit(env::current_account_id(), false)
            .then(
                ext_ft::ext(b_contract.clone()).storage_deposit(env::current_account_id(), false),
            );

        let mut tokens = UnorderedMap::new(b"t");
        tokens.insert(&a_contract, &TokenInfo::default());
        tokens.insert(&b_contract, &TokenInfo::default());

        let mut this = Self {
            owner_id,
            tokens,
            tokens_ratio: 0.0,
        };
        this.update_meta();
        this.update_tokens_ratio();
        this
    }

    fn update_meta(&mut self) {
        for (token_addr, _info) in self.tokens.iter() {
            // call cross-contract function on Token's contract to get metadata
            ext_ft::ext(token_addr.clone())
                .ft_metadata()
                .then(Self::ext(env::predecessor_account_id()).ft_metadata_callback(&token_addr));
        }
    }

    fn update_tokens_ratio(&mut self) {
        for (token_addr, _info) in self.tokens.iter() {
            ext_ft::ext(token_addr.clone())
                .ft_balance_of(env::current_account_id())
                .then(Self::ext(token_addr.clone()).ft_balance_of_callback(&token_addr));
        }

        self.tokens_ratio = 0.0;
        for (_token_addr, info) in self.tokens.iter() {
            if self.tokens_ratio == 0.0 {
                self.tokens_ratio = info.balance as f64;
            } else {
                self.tokens_ratio = self.tokens_ratio as f64 / info.balance as f64;
            }
        }
    }

    pub fn info(&self) -> String {
        let mut res = "".to_string();
        for (token_addr, token_info) in &self.tokens {
            res.push_str(
                format!(
                    "Token address: {}. Token name: {}. Decimals: {}. Ticker: TBD. Balance: {}; ",
                    token_addr, token_info.name, token_info.decimals, token_info.balance
                )
                .as_str(),
            );
        }
        res.push_str(format!("Tokens ratio: {}", self.tokens_ratio).as_str());
        res
    }

    pub fn swap(&mut self, buy_token_addr: AccountId, sell_token_addr: AccountId, amount: u128) {
        // basic error handling
        if buy_token_addr.eq(&sell_token_addr) {
            panic!("Tokens for swap must be different")
        }
        if amount == 0 {
            panic!("Requested amount for sell must be greater than 0")
        }
        for token in [&buy_token_addr, &sell_token_addr] {
            if self.tokens.get(token).is_none() {
                panic!("Token {} is not supported", token.as_str())
            }
        }

        // fill the pool
        ext_ft::ext(sell_token_addr.clone()).ft_transfer(
            env::current_account_id(),
            U128(amount),
            None,
        );
        // get balances and convert amounts to the same number of decimals
        let TokenInfo {
            name: _,
            decimals,
            balance,
        } = self.tokens.get(&sell_token_addr).unwrap();
        let x = balance / 10_u128.pow(decimals as u32);
        let TokenInfo {
            name: _,
            decimals,
            balance,
        } = self.tokens.get(&buy_token_addr).unwrap();
        let y = balance / 10_u128.pow(decimals as u32);
        let k = x * y;

        // y - (k / (x + dx))
        let buy_amount = U128(y - (k / (x + amount)));
        // transfer buy_token to initializer of swap operation
        ext_ft::ext(sell_token_addr).ft_transfer(env::current_account_id(), buy_amount, None);
    }

    pub fn deposit(&mut self, token_addr: AccountId, amount: u128) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Only the owner of the contract can call this method"
        );
        ext_ft::ext(token_addr).ft_transfer(env::current_account_id(), U128(amount), None);
        self.update_tokens_ratio();
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
}
