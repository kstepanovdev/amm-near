use std::vec;

use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::serde_json::{self, json};
use near_sdk::{
    env, ext_contract, log, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, Promise,
    PromiseOrValue,
};

pub const GAS: Gas = Gas(300_000_000_000_000);
const MIN_STORAGE: Balance = 1_000_000_000_000_000_000_000_000;

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct AMM {
    pub owner_id: AccountId,
    pub tokens: UnorderedMap<AccountId, TokenInfo>,
    pub tokens_ratio: f64,
    pub k: u128,
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
    fn ft_transfer(&self, receiver: AccountId, amount: U128, memo: Option<String>) -> Promise;
    fn ft_balance_of(&self, account_id: AccountId) -> Promise;
    fn storage_deposit(&self, account_id: AccountId, registration_only: bool) -> Promise;
}

#[near_bindgen]
impl AMM {
    #[private]
    pub fn ft_balance_of_callback(
        &mut self,
        account_id: &AccountId,
        #[callback_unwrap] balance: U128,
    ) {
        let mut token_info = self
            .tokens
            .get(account_id)
            .unwrap_or_else(|| panic!("The token {}, is not supported", account_id));
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
}

#[near_bindgen]
impl AMM {
    #[init]
    pub fn new(owner_id: AccountId, a_contract: AccountId, b_contract: AccountId) -> Self {
        // create wallet A for the AMM's account
        ext_ft::ext(a_contract.clone())
            .with_attached_deposit(MIN_STORAGE)
            .storage_deposit(env::current_account_id(), false);
        // create wallet B for the AMM's account
        ext_ft::ext(b_contract.clone())
            .with_attached_deposit(MIN_STORAGE)
            .storage_deposit(env::current_account_id(), false);

        let mut tokens = UnorderedMap::new(b"t");
        tokens.insert(&a_contract, &TokenInfo::default());
        tokens.insert(&b_contract, &TokenInfo::default());

        let mut this = Self {
            owner_id,
            tokens,
            tokens_ratio: 0.0,
            k: 0,
        };
        this.get_metadata();
        this
    }

    fn get_metadata(&mut self) {
        for (token_addr, _info) in self.tokens.iter() {
            // call cross-contract function on Token's contract to get metadata
            ext_ft::ext(token_addr.clone())
                .ft_metadata()
                .then(Self::ext(env::current_account_id()).ft_metadata_callback(&token_addr));
        }
    }

    fn update_tokens_ratio(&mut self) {
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
                    "Token address: {}. Token name: {}. Decimals: {}. Ticker: TBD. Balance: {:?}; ",
                    token_addr, token_info.name, token_info.decimals, token_info.balance
                )
                .as_str(),
            );
        }
        res.push_str(format!("Tokens ratio: {}", self.tokens_ratio).as_str());
        res
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for AMM {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let amount = u128::from(amount);

        // Get tokens' accounts.
        let buy_token = AccountId::new_unchecked(msg);
        let sell_token = loop {
            if let Some(token_id) = self.tokens.keys().next() {
                if token_id != buy_token {
                    break token_id;
                }
            } else {
                panic!("Second token account cannot be found")
            }
        };

        if sender_id == self.owner_id {
            let sell_token_balance = self.tokens.get(&sell_token).unwrap().balance + amount;
            let buy_token_balance = self.tokens.get(&buy_token).unwrap().balance;

            self.tokens.get(&sell_token).unwrap().balance = sell_token_balance;
            self.k = buy_token_balance * sell_token_balance;
        } else {
            // (x + a)(y - b) = xy
            // x = sell_token_balance, y = buy_token_balance, a = amount, b = unknown var
            // b = ya / (x + a)
            // b = buy_token * amount / (x + amount)

            // Thus,
            // buy_token_balance -= b
            // sell_token_balance += amount
            // k aka xy remains the same

            let TokenInfo {
                name: _,
                decimals: _,
                balance,
            } = self.tokens.get(&sell_token).unwrap();
            let x = balance;

            let TokenInfo {
                name: _,
                decimals: _,
                balance,
            } = self.tokens.get(&buy_token).unwrap();
            let y = balance;

            // y - (k / (x + dx)). Then return decimals.
            let b = (amount * y) / (amount + x);

            // update balances
            self.tokens.get(&sell_token).unwrap().balance += amount;
            self.tokens.get(&buy_token).unwrap().balance -= b;

            // transfer buy_token to initializer of swap operation
            ext_ft::ext(buy_token).with_attached_deposit(1).ft_transfer(
                sender_id,
                U128::from(b),
                None,
            );
        }
        PromiseOrValue::Value(U128::from(0_u128))
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
