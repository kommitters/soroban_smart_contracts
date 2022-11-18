#![no_std]

use soroban_sdk::{contractimpl, contracttype, Env, BigInt, AccountId, Symbol, Vec};

use soroban_auth::{Identifier, Signature};

mod token {
    soroban_sdk::contractimport!(file = "./soroban_token_spec.wasm");
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Name,
    NodeId,
    ChildNodes // Vec<Node>
}

#[derive(Clone)]
#[contracttype]
pub enum Node {
    Name,   // 10 character descriptive name
    Address,    // Stellar public key(AccountId) or Contract ID (BytesN<32>)
    Percentage  // Corresponding percentage of the donation
}

fn set_children(env: &Env) {

}
pub struct CascadeDonationContract;

pub trait CascadeDonationContractTrait {
    fn initialize(env: Env);
    fn donate(env: Env, amount: BigInt);
    fn children(env: Env, new_children: Vec<Node>);
}

#[contractimpl]
impl CascadeDonationContractTrait for CascadeDonationContract {
    fn initialize(env: Env) {
    }

    fn donate(env: Env, amount: BigInt) {
        /*
        *  1. Extract the "amount" from the donator account, into contract account
        *  2. Iterate trhough the children, and:
        *      if child.address == AccountId:
        *           transfer -> AccountId, equivalent percentage
        *      else (BytesN<32>):
        *           Create instance of a CascadeDonationContract using the address
        *           use the donation behavior to send the "amount" to the child contract
        */
    }

    fn children(env: Env, new_children: Vec<Node>) {
        set_children(&env);
    }
}

#[cfg(test)]
mod test;
