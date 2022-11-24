#![cfg(test)]

use super::{CascadeDonationContract, CascadeDonationContractClient, Identifier, Node};
use soroban_sdk::{symbol, vec, Env, testutils::{Accounts}, BigInt, IntoVal, BytesN, Vec};
use soroban_auth::{Signature};


use crate::token::{self, TokenMetadata};

mod cascade_donation_contract {
    soroban_sdk::contractimport!(
        file = "./target/wasm32-unknown-unknown/release/cascade_donation.wasm"
    );
}

extern crate std;

fn create_and_init_token_contract(env: &Env, admin_id: &Identifier) -> (BytesN<32>, token::Client) {
    let token_id = env.register_contract_token(None);
    let token_client = token::Client::new(&env, &token_id);

    token_client.init(
        &admin_id,
        &TokenMetadata {
            name: "USD Coin".into_val(&env),
            symbol: "USDC".into_val(&env),
            decimals: 7,
        },
    );

    (token_id, token_client)
}

#[test]
fn basic_donation_without_cascade() {
    /*
        [EXAMPLE]
        MAIN PROJECT
        |-   dependencie_1
        |-   dependencie_2

        Expected workflow
        1. Donate 1000 to MAIN_PROJECT
        2. The MAIN PROJECT receives the donation
        3. Auto donate to dependencie_1 with 10 percentege with a xfer
        4. Auto donate to dependencie_2 with 30 percentege with a xfer

        Expected Result = {
            MAIN_PROJECT -> 600
            dependencie_1 -> 100
            dependencie_2 -> 300
        }
    */

    let env = Env::default();

    let admin = env.accounts().generate();
    let admin_id = Identifier::Account(admin.clone());

    let donator = env.accounts().generate();
    let donator_id = Identifier::Account(donator.clone());

    // CONTRACT
    let contract_id = env.register_contract(None, CascadeDonationContract);
    let contract_client = CascadeDonationContractClient::new(&env, &contract_id);

    // CREATE TOKEN CONTRACT
    let (token_id, token_client) = create_and_init_token_contract(&env, &admin_id);

    // CHILDREN ACCOUNTS
    let dependencie_1 = env.accounts().generate();
    let dependencie_1_id = Identifier::Account(dependencie_1.clone());

    let dependencie_2 = env.accounts().generate();
    let dependencie_2_id = Identifier::Account(dependencie_2.clone());

    // CHILDREN NODES
    let child_1 =
        Node {
            address: soroban_sdk::Address::Account(dependencie_1.clone()),
            name: symbol!("dep_1"),
            percentage: 10
        };

    let child_2 =
        Node {
            address: soroban_sdk::Address::Account(dependencie_2.clone()),
            name: symbol!("dep_2"),
            percentage: 30
        };

    let mut children: Vec<Node> = vec![&env];
    children.push_back(child_1);
    children.push_back(child_2);

    contract_client.initialize(&token_id, &children);

    // FUND DONATOR ACCOUNT
    token_client.with_source_account(&admin).mint(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &donator_id,
        &BigInt::from_u32(&env, 2000)
    );

    token_client.with_source_account(&donator).approve(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &Identifier::Contract(contract_id.clone()),
        &BigInt::from_u32(&env, 1000)
    );

    contract_client.with_source_account(&donator).donate(&BigInt::from_u32(&env, 1000), &donator_id);

    std::println!("======= DONATOR BALANCE ========: {:?}", token_client.balance(&donator_id));
    std::println!("======= CONTRACT BALANCE ========: {:?}", token_client.balance(&Identifier::Contract(contract_id.clone())));
    std::println!("======= dependencie_1 BALANCE ========: {:?}", token_client.balance(&dependencie_1_id));
    std::println!("======= dependencie_2 BALANCE ========: {:?}", token_client.balance(&dependencie_2_id));
    std::println!("==================================");

    assert_eq!(
        token_client.balance(&Identifier::Contract(contract_id.clone())),
        &BigInt::from_u32(&env, 600),
        "Main project gets the correct balance"
    );

    assert_eq!(
        token_client.balance(&dependencie_1_id),
        &BigInt::from_u32(&env, 100),
        "Dependencie 1 receives the correct balance"
    );

    assert_eq!(
        token_client.balance(&dependencie_2_id),
        &BigInt::from_u32(&env, 300),
        "Dependencie 2 receives the correct balance"
    );

}

#[test]
fn contract_with_parent_children() {
        /*
        [EXAMPLE]
        MAIN PROJECT
        |-   dependencie_1
        |-   dependencie_2
        |--     sub_dependencie_1
        |--     sub_dependencie_2

        Expected workflow
        1. Donate 1000 to MAIN PROJECT
        2. The MAIN PROJECT receives the donation
        3. Auto donate to dependencie_1 with a xfer
        4. Auto donate to dependencie_2 with a donation invocation (Should be a contract since is a child with sub childs)
        5. The dependencie_2 receives the donation
        6. Auto donate to sub_dependencie_1 with a xfer
        7. Auto donate to sub_dependencie_2 with a xfer

        Expected Result = {
            MAIN_PROJECT -> 600
            dependencie_1 -> 200
            CHILD PARENT -> 120
            sub_dependencie_1 -> 40
            sub_dependencie_2 -> 40
        }
    */

    let env = Env::default();

    // USERS
    let admin = env.accounts().generate();
    let admin_id = Identifier::Account(admin.clone());

    let donator = env.accounts().generate();
    let donator_id = Identifier::Account(donator.clone());

    // PARENT CONTRACT (PARENT PROJECT)
    let contract_id = env.register_contract(None, CascadeDonationContract);
    let contract_client = CascadeDonationContractClient::new(&env, &contract_id);

    // PARENT PROJECT CHILDREN ACCOUNTS
    let dependencie_1 = env.accounts().generate();
    let dependencie_1_id = Identifier::Account(dependencie_1.clone());

    // PARENT CHILD CONTRACT (CHILD CONTRACT)
    let child_contract_id = env.register_contract_wasm(None, cascade_donation_contract::WASM);
    let child_contract_client = CascadeDonationContractClient::new(&env, &child_contract_id);

    // SUB PROJECT 2 CHILDREN
    let sub_dependencie_1 = env.accounts().generate();
    let sub_dependencie_1_id = Identifier::Account(sub_dependencie_1.clone());

    let sub_dependencie_2 = env.accounts().generate();
    let sub_dependencie_2_id = Identifier::Account(sub_dependencie_2.clone());

    // CREATE TOKEN CONTRACT
    let (token_id, token_client) = create_and_init_token_contract(&env, &admin_id);

    // CHILD PARENT CHILDREN
    let mut parent1_children: Vec<Node> = vec![&env];
    let parent_1_child_1 =
        Node {
            address: soroban_sdk::Address::Account(sub_dependencie_1.clone()),
            name: symbol!("subdep_1"),
            percentage: 20
        };

    let parent_1_child_2 =
        Node {
            address: soroban_sdk::Address::Account(sub_dependencie_2.clone()),
            name: symbol!("subdep_2"),
            percentage: 20
        };

    parent1_children.push_back(parent_1_child_1);
    parent1_children.push_back(parent_1_child_2);
    // END CHILD PARENT CHILDREN

    child_contract_client.initialize(&token_id, &parent1_children);
    std::println!("======= CHILD CONTRACT CHILDREN ========: {:?}", child_contract_client.g_children());
    std::println!("========================================:");

    //PARENT CHILDREN
    let child_parent_1 =
        Node {
            address: soroban_sdk::Address::Contract(child_contract_id.clone()),
            name: symbol!("c_parent_1"),
            percentage: 20
        };

    let child_1 =
        Node {
            address: soroban_sdk::Address::Account(dependencie_1.clone()),
            name: symbol!("dep_1"),
            percentage: 20
        };
    // END CHILDREN

    let mut children: Vec<Node> = vec![&env];
    children.push_back(child_1);
    children.push_back(child_parent_1);

    contract_client.initialize(&token_id, &children);
    std::println!("======= MAIN CONTRACT CHILDREN ========: {:?}", contract_client.g_children());

    // FUND DONATOR ACCOUNT
    token_client.with_source_account(&admin).mint(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &donator_id,
        &BigInt::from_u32(&env, 2000)
    );

    std::println!("======= DONATOR BALANCE ========: {:?}", token_client.balance(&donator_id));
    std::println!("======= CONTRACT BALANCE ========: {:?}", token_client.balance(&Identifier::Contract(contract_id.clone())));
    std::println!("======= dependencie_1 BALANCE ========: {:?}", token_client.balance(&dependencie_1_id));
    std::println!("======= CHILD PARENT CONTRACT BALANCE ========: {:?}", token_client.balance(&Identifier::Contract(child_contract_id.clone())));
    std::println!("======= sub_dependencie_1 BALANCE ========: {:?}", token_client.balance(&sub_dependencie_1_id));
    std::println!("======= sub_dependencie_2 BALANCE ========: {:?}", token_client.balance(&sub_dependencie_2_id));
    std::println!("==================================");

    token_client.with_source_account(&donator).approve(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &Identifier::Contract(contract_id.clone()),
        &BigInt::from_u32(&env, 1000)
    );

    contract_client.with_source_account(&donator).donate(&BigInt::from_u32(&env, 1000), &donator_id);

    assert_eq!(
        token_client.balance(&Identifier::Contract(contract_id.clone())),
        &BigInt::from_u32(&env, 600),
        "Main project gets the correct balance"
    );

    assert_eq!(
        token_client.balance(&dependencie_1_id),
        &BigInt::from_u32(&env, 200),
        "Dependencie 1 receives the correct balance"
    );

    assert_eq!(
        token_client.balance(&Identifier::Contract(child_contract_id.clone())),
        &BigInt::from_u32(&env, 120),
        "Parent Child receives the correct balance"
    );

    assert_eq!(
        token_client.balance(&sub_dependencie_1_id),
        &BigInt::from_u32(&env, 40),
        "Sub Dependencie 1 receives the correct balance"
    );

    assert_eq!(
        token_client.balance(&sub_dependencie_2_id),
        &BigInt::from_u32(&env, 40),
        "Sub Dependencie 2 receives the correct balance"
    );
}
