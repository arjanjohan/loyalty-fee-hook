#![cfg_attr(not(feature = "export-abi"), no_main)]

#[cfg(feature = "export-abi")]
fn main() {
    fee_logic::print_abi("MIT", "pragma solidity ^0.8.23;");
}
