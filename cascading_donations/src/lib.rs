#![no_std]

use soroban_sdk::{contractimpl, contracttype, Env, BigInt, AccountId, BytesN, Vec, ConversionError, Symbol, Address};

use soroban_auth::{Identifier, Signature};

mod token {
    soroban_sdk::contractimport!(file = "./soroban_token_spec.wasm");
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Name,
    Admin,
    TContract,
    ChildNodes // Vec<Node>
}

#[derive(Clone)]
#[contracttype]
pub struct Node {
    name: Symbol,   // 10 character descriptive name
    address: Address,    // Stellar public key(AccountId) or Contract ID (BytesN<32>)
    percentage: u32  // Corresponding percentage of the donation
}

// CHILDREN
fn get_children(env: &Env) -> Vec<Node> {
    let key = DataKey::ChildNodes;
    env.data().get(key).unwrap().unwrap()
}

fn set_children(env: &Env, new_children: &Vec<Node>) {
    env.data().set(DataKey::ChildNodes, new_children);
}

// TOKEN CONTRACT
fn get_token_contract_id(env: &Env) -> BytesN<32> {
    let key = DataKey::TContract;
    env.data().get(key).unwrap().unwrap()
}

fn set_token_contract_id(e: &Env, token_id: &BytesN<32>) {
    e.data().set(DataKey::TContract, token_id);
}

// ADMIN
fn get_admin_id(env: &Env) -> Identifier {
    let key = DataKey::Admin;
    env.data().get(key).unwrap().unwrap()
}

fn set_admin_id(e: &Env, admin_id: &Identifier) {
    e.data().set(DataKey::Admin, admin_id);
}

fn donate_to_child(env: &Env, node: &Node) {
    // APPROVE SIGN

    // XFER
}

fn apply_children_donations(env: &Env) {
    /*
    *  Iterate trhough the children, and:
    *      if child.address == AccountId:
    *           transfer -> AccountId, equivalent percentage
    *      else (BytesN<32>):
    *           Create instance of a CascadeDonationContract using the address
    *           use the donation behavior to send the "amount" to the child contract
    */
    // let children = get_children(&env);

    // for child in children {
    //     match child {
    //         Ok(node) => donate_to_child(env, &node),
    //         Err(error) => panic!("Problem reading the node: {:?}", error),
    //     }
    // }
}

fn apply_donation(env: &Env, amount: &BigInt, donator: &Identifier) {
    //Extract the "amount" from the donator account, into contract account

    let tc_id = get_token_contract_id(&env);
    let client = token::Client::new(&env, &tc_id);

    let contract = env.current_contract();
    let contract_identifier = Identifier::Contract(contract.clone());

    client.xfer_from(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &donator,
        &contract_identifier,
        &amount
    );

    //apply_children_donations(&env);
}

pub struct CascadeDonationContract;

pub trait CascadeDonationContractTrait {
    fn initialize(env: Env, tc_id: BytesN<32>, admin_id: Identifier, children: Vec<Node>);
    fn donate(env: Env, amount: BigInt, donator: Identifier);
    fn s_children(env: Env, new_children: Vec<Node>);
}

#[contractimpl]
impl CascadeDonationContractTrait for CascadeDonationContract {
    fn initialize(env: Env, tc_id: BytesN<32>, admin_id: Identifier, children: Vec<Node>) {
        set_token_contract_id(&env, &tc_id);
        set_admin_id(&env, &admin_id);
        set_children(&env, &children)
    }

    fn donate(env: Env, amount: BigInt, donator: Identifier) {
        apply_donation(&env, &amount, &donator);
    }

    fn s_children(env: Env, new_children: Vec<Node>) {
        set_children(&env, &new_children);
    }
}

#[cfg(test)]
mod test;
