use std::vec;

use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::{env, ext_contract, near_bindgen, AccountId, Gas, PanicOnDefault, Promise};

pub const GAS: Gas = Gas(5_000_000_000_000);

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
}

#[near_bindgen]
impl AMM {
    #[init]
    pub fn new(owner_id: AccountId, a_contract: AccountId, b_contract: AccountId) -> Self {
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
                .ft_balance_of(token_addr.clone())
                .then(Self::ext(env::predecessor_account_id()).ft_balance_of_callback(&token_addr));
        }

        let mut ratio = 0.0;
        for (_token_addr, info) in self.tokens.iter() {
            if ratio == 0.0 {
                ratio = info.balance as f64;
            } else {
                ratio = ratio as f64 / info.balance as f64;
            }
        }
        self.tokens_ratio = ratio;
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

    // pub fn swap(&mut self, buy_token_addr: AccountId, sell_token_addr: AccountId, amount: u128) {
    //     let mut buy_token = self
    //         .tokens
    //         .get(&buy_token_addr)
    //         .expect("Token not supported");
    //     let mut sell_token = self
    //         .tokens
    //         .get(&sell_token_addr)
    //         .expect("Token not supported");
    //     let pool_owner_id = &self.owner_id;
    //     let user_account_id = env::predecessor_account_id();

    //     let x = sell_token.0.internal_unwrap_balance_of(pool_owner_id);
    //     let y = buy_token.0.internal_unwrap_balance_of(pool_owner_id);

    //     let k = x * y;

    //     sell_token
    //         .0
    //         .internal_transfer(&user_account_id, pool_owner_id, amount, None);

    //     // operations with a floating pointer won't be implemented due to my lack of experience of such operations
    //     // y - (k / (x + dx))
    //     let buy_amount = y - (k / (x + amount));

    //     buy_token
    //         .0
    //         .internal_transfer(pool_owner_id, &user_account_id, buy_amount, None);

    //     self.tokens.insert(&buy_token_addr, &buy_token);
    //     self.tokens.insert(&sell_token_addr, &sell_token);

    //     let new_ratio = buy_token.0.total_supply as f64 / sell_token.0.total_supply as f64;
    //     sell_token.2.update(new_ratio);
    //     buy_token.2.update(new_ratio);
    // }

    // pub fn deposit(
    //     &mut self,
    //     token_a_addr: AccountId,
    //     token_b_addr: AccountId,
    //     amount_a: u128,
    //     amount_b: u128,
    // ) {
    //     assert_eq!(
    //         env::predecessor_account_id(),
    //         self.owner_id,
    //         "Only the owner may call this method"
    //     );

    //     let payer = env::predecessor_account_id();

    //     let mut token_a = self.tokens.get(&token_a_addr).expect("Token not supported");
    //     let mut token_b = self.tokens.get(&token_b_addr).expect("Token not supported");
    //     token_a
    //         .0
    //         .internal_transfer(&payer, &self.owner_id, amount_a, None);
    //     token_b
    //         .0
    //         .internal_transfer(&payer, &self.owner_id, amount_b, None);

    //     self.tokens.insert(&token_a_addr, &token_a);
    //     self.tokens.insert(&token_b_addr, &token_b);
    // }
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
