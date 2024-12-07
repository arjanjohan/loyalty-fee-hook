// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(all(not(feature = "std"), not(feature = "export-abi")), no_main)]
extern crate alloc;

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    evm,
    prelude::entrypoint,
    stylus_proc::{public, sol_storage, SolidityError},
};

use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;

/// The currency data type.
pub type Currency = Address;

// sol! {
//     /// Emitted when the amount of input tokens for an exact-output swap
//     /// is calculated.
//     #[allow(missing_docs)]
//     event AmountInCalculated(
//         uint256 amount_out,
//         address input,
//         address output,
//         bool zero_for_one
//     );

//     /// Emitted when the amount of output tokens for an exact-input swap
//     /// is calculated.
//     #[allow(missing_docs)]
//     event AmountOutCalculated(
//         uint256 amount_in,
//         address input,z
//         address output,
//         bool zero_for_one
//     );
// }

sol! {
    /// Indicates a custom error.
    #[derive(Debug)]
    #[allow(missing_docs)]
    error LoyaltyFeeCustomError();
}

#[derive(SolidityError, Debug)]
pub enum Error {
    /// Indicates a custom error.
    CustomError(LoyaltyFeeCustomError),
}

sol_storage! {
    #[entrypoint]
    struct FeeLogic {

        Tier[] tiers;
        
        mapping(address => uint256) points;
        mapping(address => uint256) lastActivityBlock;
        uint256 totalPoints;
        uint24 baseFee;
        uint256 expirationBlocks;
    }

    struct Tier {
        uint256 threshold;
        uint24 discount; // in basis points (10000 = 100%)
    }
}

/// Interface of an [`FeeLogic`] contract.
///
/// NOTE: The contract's interface can be modified in any way.
pub trait IFeeLogic {
    fn get_total_points(& self) -> Result<U256, Error>;

    fn get_user_points(&self, user: Address) -> Result<U256, Error>;

    fn get_fee(&mut self, block_number: U256, user: Address) -> Result<U256, Error>;

    fn update_points(
        &mut self,
        user: Address,
        zero_for_one: bool,
        amount_specified: U256,
        delta_amount_0: U256,
        currency_0: Address,
        currency_1: Address
    ) -> Result<(), Error>;
}

/// Declare that [`FeeLogic`] is a contract
/// with the following external methods.
#[public]
impl IFeeLogic for FeeLogic {
    fn get_total_points(& self) -> Result<U256, Error> {
        Ok(self.totalPoints.clone())
    }

    fn get_user_points(&self, user: Address) -> Result<U256, Error> {
        Ok(U256::from(self.points.get(user)))
    }

    fn get_fee(&mut self, block_number: U256, user: Address) -> Result<U256, Error> {
        // Reset points if user has not been active for the last expiration blocks
        if block_number - self.lastActivityBlock.get(user) > self.expirationBlocks.clone() {
            
            let expired_points = self.points.replace(user, U256::ZERO);
            self.totalPoints.checked_sub(expired_points);

        }

        // Calculate and return fee for user
        let fee = self.calculate_fee(user)?;
        Ok(U256::from(fee))
    }

    fn update_points(
        &mut self,
        user: Address,
        zero_for_one: bool,
        amount_specified: U256,
        delta_amount_0: U256,
        currency_0: Address,
        currency_1: Address
    ) -> Result<(), Error> {
        Ok(())
    }
}
impl FeeLogic {
    fn calculate_fee(&self, user: Address) -> Result<U256, Error> {
        let user_points = self.points.get(user);
        let base_fee = U256::from(*self.baseFee); // TODO: Set base fee
        // Check tiers from highest to lowest to find applicable discount
        for i in 0..self.tiers.len() {
            if user_points >= *self.tiers.get(i).unwrap().threshold {
                
                let discount = U256::from(*self.tiers.get(i).unwrap().discount);
                let discounted_amount = (base_fee * discount) / U256::from(10000);
                return Ok(base_fee - discounted_amount);
            }
        }

        // No tier matched, return base fee
        Ok(base_fee)
    }
}
