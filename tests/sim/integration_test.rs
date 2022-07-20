// use crate::utils::{init, AMM_ID};
// use near_sdk::{json_types::U128, env::storage_byte_cost};
// use near_sdk_sim::{call, view, DEFAULT_GAS};

// #[test]
// fn check_info() {
//     let (_, _, _, amm, alice, _) = init();

//     // let expected_info = "";
//     // let gathered_info = view!(
//     //     amm.info()
//     // );
//     // .unwrap_json();

//     alice.call(
//         amm.account_id(),
//         "info",
//         &[0],
//         DEFAULT_GAS,
//         storage_byte_cost()
//     )
//     .assert_success();

//     // assert_eq!(expected_info, gathered_info);
// }

// #[test]
// fn check_deposit_from_root() {
//     let (
//         root,
//         a,
//         b,
//         amm,
//         _,
//         _
//     ) = init();

//     call!(
//         root,
//         amm.deposit(a, b, 1, 10)
//     )
//     .assert_success();
// }

// #[test]
// #[should_panic(expected = "Only the owner may call this method")]
// fn check_deposit_from_random() {
//     let (_, a, b, amm, alice, _) = init();

//     call!(
//         alice,
//         amm.deposit(a, b, 1, 10)
//     )
//     .expect()
// }

// #[test]
// fn check_swap() {
//     let init_alice_a_token_amount = 600_u128;
//     let init_bob_b_token_amount = 200_u128;

//     let transfer_to_alice_amount_b = 200_000_u128;
//     let (
//         root,
//         a_token,
//         b_token,
//         amm,
//         alice,
//         bob
//      ) = init();

//     // init AMM contract and deposit it with tokens, thus setting up the K (k = a * b) ratio.
//     // E.g. let it be 2700.

//     call!(
//         root,
//         amm.deposit(a_token, b_token, 900, 300),
//         gas = DEFAULT_GAS
//     )
//     .assert_success();

//     // add some tokens for alice and bob
//     call!(
//         root,
//         a_token.ft_transfer(alice.account_id(), init_alice_a_token_amount.into(), None),
//         deposit = 1
//     )
//     .assert_success();
//     call!(
//         root,
//         b_token.ft_transfer(bob.account_id(), init_bob_b_token_amount.into(), None),
//         deposit = 1
//     )
//     .assert_success();

//     // // Open storage in AMM for Alice
//     // call!(
//     //     root,
//     //     amm.storage_deposit(ft_a.account_id(), alice.account_id(), None),
//     //     deposit = near_sdk::env::storage_byte_cost() * 125
//     // )
//     // .assert_success();
//     // call!(
//     //     root,
//     //     amm.storage_deposit(ft_b.account_id(), alice.account_id(), None),
//     //     deposit = near_sdk::env::storage_byte_cost() * 125
//     // )
//     // .assert_success();

//     // Swap tokens
//     let sell_token = token_a.account_id();
//     let buy_token = token_b.account_id();
//     let amount = 100_u128;
//     // Pool's k = 900A * 300B = 2700
//     // expected amount = y - (k / (x + dx)) = 2700 - (2700 / (900 + 100)) = 2700 - 2.7 = 2697.3
//     let expected_amount = 1_u128;

//     call!(
//         alice,
//         amm.swap(buy_token, sell_token, amount.into())
//     ).assert_success;
//     outcome.assert_success();

//     let alice_balance_a: u128 =
//         view!(a_token.ft_balance_of(ft_a.account_id(), alice.account_id())).unwrap_json();
//     let pool_balance =
//         view!(a_token.ft_balance_of(ft_a.account_id(), root.account_id())).unwrap_json();

//     // Alice has got her tokens
//     assert_eq!(
//         alice_balance_a,
//         expected_amount
//     );

//     // Pool's balance has changed
//     assert_eq!(
//         pool_balance,
//         prev_pool_balance - expected_amount
//     );

//     // Pool's ratio has remained the same
//     todo!()
// }
