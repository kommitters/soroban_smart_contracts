#![cfg(test)]

use super::{CascadeDonationContract, CascadeDonationContractClient, Identifier, Node};
use soroban_sdk::{symbol, Env, testutils::{Accounts}, BigInt, IntoVal, BytesN, Vec, Symbol};
use soroban_auth::{Signature, testutils::ed25519};

use crate::token::{self, TokenMetadata};

extern crate std;

fn create_and_init_token_contract(env: &Env, admin_id: &Identifier) -> (BytesN<32>, token::Client) {
    let token_id = env.register_contract_token(None);
    let token_client = token::Client::new(&env, &token_id);

    token_client.init(
        &admin_id,
        &TokenMetadata {
            name: "Mmitkoin".into_val(&env),
            symbol: "MTK".into_val(&env),
            decimals: 7,
        },
    );

    (token_id, token_client)
}

#[test]
fn test() {
    let env = Env::default();

    // USERS
    // let (admin_id, admin_sign) = ed25519::generate(&env);

    // ADMIN
    let admin = env.accounts().generate();
    let admin_id = Identifier::Account(admin.clone());


    // APPROVAL USER
    let donator = env.accounts().generate();
    let donator_id = Identifier::Account(donator.clone());

    // CREATE OUR CUSTOM CONTRACT
    let contract_id = env.register_contract(None, CascadeDonationContract);
    let contract_client = CascadeDonationContractClient::new(&env, &contract_id);

    // CREATE TOKEN CONTRACT
    let (token_id, token_client) = create_and_init_token_contract(&env, &admin_id);

    let children: Vec<Node> = [];

    contract_client.initialize(&token_id, &admin_id);

    // FUND DONATOR ACCOUNT
    // let nonce = token_client.nonce(&admin_id);
    // let approval_sign = ed25519::sign(
    //     &env,
    //     &admin_sign,
    //     &token_id,
    //     symbol!("mint"),
    //     (&admin_id, &nonce, &admin_id, &BigInt::from_u32(&env, 2000)),
    // );

    // token_client.mint(admin, nonce, to, amount)

    token_client.with_source_account(&admin).mint(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &donator_id,
        &BigInt::from_u32(&env, 2000)
    );

    std::println!("======= DONATOR BALANCE ========: {:?}", token_client.balance(&donator_id));
    std::println!("======= CONTRACT BALANCE ========: {:?}", token_client.balance(&Identifier::Contract(contract_id.clone())));
    std::println!("=================================: {:?}", token_client.balance(&donator_id));

    token_client.with_source_account(&donator).approve(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &Identifier::Contract(contract_id.clone()),
        &BigInt::from_u32(&env, 1000)
    );

    contract_client.with_source_account(&donator).donate(&BigInt::from_u32(&env, 1000), &donator_id);
    std::println!("======= DONATOR BALANCE ========: {:?}", token_client.balance(&donator_id));
    std::println!("======= CONTRACT BALANCE ========: {:?}", token_client.balance(&Identifier::Contract(contract_id.clone())));
}