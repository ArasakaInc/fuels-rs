contract;

use std::revert::revert;

abi MyContract {
    fn make_transaction_fail(input: u64) -> u64;
}

const COUNTER_KEY = 0x0000000000000000000000000000000000000000000000000000000000000000;

impl MyContract for Contract {
    fn make_transaction_fail(input: u64) -> u64 {
        revert(input);
        42
    }
}