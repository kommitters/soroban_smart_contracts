#![cfg(test)]

use super::{CascadeDonationContract, CascadeDonationContractClient, Identifier, Node};
use soroban_sdk::{symbol, vec, Env, testutils::{Accounts}, BigInt, IntoVal, BytesN, Vec, Symbol};
use soroban_auth::{Signature, testutils::ed25519};

use crate::token::{self, TokenMetadata};

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
fn test() {
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
    */

    let env = Env::default();

    // let (admin_id, admin_sign) = ed25519::generate(&env);

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

    let dependencie_2 = env.accounts().generate();
    let dependencie_2_id = Identifier::Account(dependencie_2.clone());

    // PARENT CHILD CONTRACT (CHILD CONTRACT)
    let child_contract_id = env.register_contract(None, CascadeDonationContract);
    let child_contract_client = CascadeDonationContractClient::new(&env, &child_contract_id);

    // SUB PROJECT 2 CHILDREN
    let sub_dependencie_1 = env.accounts().generate();
    let sub_dependencie_1_id = Identifier::Account(sub_dependencie_1.clone());

    let sub_dependencie_2 = env.accounts().generate();
    let sub_dependencie_2_id = Identifier::Account(sub_dependencie_2.clone());

    // CREATE TOKEN CONTRACT
    let (token_id, token_client) = create_and_init_token_contract(&env, &admin_id);

    // PARENT 1 CHILDREN
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
    // END PARENT 1 CHILDREN

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

    child_contract_client.initialize(&token_id, &admin_id, &parent1_children);
    std::println!("======= CHILD CONTRACT CHILDREN ========: {:?}", child_contract_client.g_children());

    let mut children: Vec<Node> = vec![&env];
    children.push_back(child_1);
    children.push_back(child_parent_1);

    contract_client.initialize(&token_id, &admin_id, &children);
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
    std::println!("==================================");

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