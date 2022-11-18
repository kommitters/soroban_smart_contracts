#![no_std]

use soroban_sdk::{contractimpl, Env};

use soroban_auth::{Identifier, Signature};

mod token {
    soroban_sdk::contractimport!(file = "./soroban_token_spec.wasm");
}
pub struct CascadeDonationContract;

pub trait CascadeDonationContractTrait {
    fn initialize(env: Env);
}

#[cfg(test)]
mod test;
