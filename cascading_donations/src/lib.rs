#![no_std]

use soroban_sdk::{contractimpl, contracttype, symbol, vec, Env, BigInt, BytesN, Vec, Symbol, Address, RawVal, IntoVal};

use soroban_auth::{Identifier, Signature};

mod token {
    soroban_sdk::contractimport!(file = "./soroban_token_spec.wasm");
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Name,
    TkContract,
    ChildNodes // Vec<Node>
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Node {
    name: Symbol,
    address: Address,
    percentage: u32,
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
    let key = DataKey::TkContract;
    env.data().get(key).unwrap().unwrap()
}

fn set_token_contract_id(e: &Env, token_id: &BytesN<32>) {
    e.data().set(DataKey::TkContract, token_id);
}

fn donate_to_child(env: &Env, child_address: &Identifier, percentage: &u32, base_balance: &BigInt) {
    let tc_id = get_token_contract_id(&env);
    let client = token::Client::new(&env, &tc_id);

    let amount: BigInt = (base_balance * percentage) / 100;
    
    client.xfer(
        &Signature::Invoker,
        &BigInt::zero(&env),
        &child_address,
        &amount
    );
}

fn donate_to_parent_child(env: &Env, node_contract_id: &BytesN<32>, percentage: &u32, base_balance: &BigInt, donator: &Identifier) {
    donate_to_child(&env, &Identifier::Contract(node_contract_id.clone()), &percentage, &base_balance);
    
    let raw_donator = donator.into_val(&env);

    let args: Vec<RawVal> = vec![
        &env,
        raw_donator
    ];
    
    env.invoke_contract(&node_contract_id, &symbol!("donate_ch"), args)
}

fn apply_donation_type(env: &Env, child: &Node, base_balance: &BigInt, donator: &Identifier) {
    match &child.address {
        Address::Contract(contract_id) => donate_to_parent_child(&env, &contract_id, &child.percentage, &base_balance, &donator),
        Address::Account(account_id) =>  donate_to_child(&env, &Identifier::Account(account_id.clone()), &child.percentage, &base_balance),
    }
}

fn apply_children_donations(env: &Env, base_balance: &BigInt, donator: &Identifier) {
    for child in get_children(&env) {
        match child {
            Ok(node) => apply_donation_type(env, &node, &base_balance, &donator),
            Err(error) => panic!("Problem reading the node: {:?}", error),
        }
    }
}

fn apply_main_donation(env: &Env, donator: &Identifier, amount: &BigInt) {
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

    let contract_balance = client.balance(&contract_identifier);

    apply_children_donations(&env, &contract_balance, &donator);
}
pub struct CascadeDonationContract;

pub trait CascadeDonationContractTrait {
    fn initialize(env: Env, tc_id: BytesN<32>, children: Vec<Node>);
    fn donate(env: Env, amount: BigInt, donator: Identifier);
    fn donate_ch(env: Env, donator: Identifier);
    fn s_children(env: Env, new_children: Vec<Node>);
    fn g_children(env: Env) -> Vec<Node>;
}

#[contractimpl]
impl CascadeDonationContractTrait for CascadeDonationContract {
    fn initialize(env: Env, tc_id: BytesN<32>, children: Vec<Node>) {
        set_token_contract_id(&env, &tc_id);
        set_children(&env, &children)
    }

    fn donate(env: Env, amount: BigInt, donator: Identifier) {
        apply_main_donation(&env, &donator, &amount);
    }

    fn donate_ch(env: Env, donator: Identifier) {
        let tc_id = get_token_contract_id(&env);
        let client = token::Client::new(&env, &tc_id);

        let contract = env.current_contract();
        let contract_identifier = Identifier::Contract(contract.clone());

        let contract_balance = client.balance(&contract_identifier);

        apply_children_donations(&env, &contract_balance, &donator);
    }

    fn s_children(env: Env, new_children: Vec<Node>) {
        set_children(&env, &new_children);
    }

    fn g_children(env: Env) -> Vec<Node> {
        get_children(&env)
    }
}

#[cfg(test)]
mod test;
