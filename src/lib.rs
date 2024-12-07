// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(all(not(feature = "std"), not(feature = "export-abi")), no_main)]
extern crate alloc;

use std::borrow::BorrowMut;

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    block,
    prelude::entrypoint,
    stylus_proc::{public, sol_storage, SolidityError},
};

use alloy_primitives::{Address, I256, U256};
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

        mapping(address => mapping(address => uint256)) points;
        mapping(address => mapping(address => uint256)) last_activity_block;
        uint256 base_fee;
        uint256 expiration_blocks;
    }


    struct Tier {
        uint256 threshold;
        uint256 discount; // in basis points (10000 = 100%)
    }
}

/// Interface of an [`FeeLogic`] contract.
///
/// NOTE: The contract's interface can be modified in any way.
pub trait IFeeLogic {
    fn init(&mut self);

    fn get_user_points(&self, user: Address, currency_1: Address) -> Result<U256, Error>;

    fn get_fee(&mut self, user: Address, currency_1: Address) -> Result<U256, Error>;

    fn update_points(
        &mut self,
        user: Address,
        zero_for_one: bool,
        amount_specified: I256,
        delta_amount_0: I256,
        currency_0: Address,
        currency_1: Address,
    ) -> Result<(), Error>;
}

/// Declare that [`FeeLogic`] is a contract
/// with the following external methods.
#[public]
impl IFeeLogic for FeeLogic {
    fn init(&mut self) {
        self.base_fee.set(U256::from(50000));
        self.expiration_blocks.set(U256::from(100));
        for i in 0..3 {
            let threshold = U256::from(1000000000000000000);
            let tier = Tier { threshold: threshold, discount: U256::from(5000) };
            let x = self.tiers.setter(i).insert(tier);
        }
    }

    fn get_user_points(&self, user: Address, currency_1: Address) -> Result<U256, Error> {
        Ok(U256::from(self.points.get(currency_1).get(user)))
    }

    fn get_fee(&mut self, user: Address, currency_1: Address) -> Result<U256, Error> {
        let block_number = U256::from(block::number());
        // Reset points if user has not been active for the last expiration blocks
        if block_number - self.last_activity_block.get(currency_1).get(user) > self.expiration_blocks.clone() {
            let points_map = self.points.borrow_mut();
            points_map.setter(currency_1).setter(user).set(U256::ZERO);
        }

        // Calculate and return fee for user
        let fee = self.calculate_fee(user)?;
        Ok(U256::from(fee))
    }

    fn update_points(
        &mut self,
        user: Address,
        zero_for_one: bool,
        amount_specified: I256,
        delta_amount_0: I256,
        _currency_0: Address,
        currency_1: Address,
    ) -> Result<(), Error> {
        // Calculate points earned with swap
        let points_earned =
            self.calculate_points_earned(zero_for_one, amount_specified, delta_amount_0)?;

        // Update points
        let points = self.points.get(user);
        let new_points = points.get(currency_1).checked_add(points_earned).unwrap();
        let points_map = &mut self.points.borrow_mut();
        points_map.setter(currency_1).setter(user).set(new_points);

        // Update last activity block
        let block_number = U256::from(block::number());
        let last_activity_block = &mut self.last_activity_block.borrow_mut();
        last_activity_block.setter(currency_1).setter(user).set(block_number);
        Ok(())
    }
}
impl FeeLogic {
    fn calculate_fee(&self, user: Address) -> Result<U256, Error> {
        let user_points = self.points.get(user);
        let base_fee = U256::from(*self.base_fee); // TODO: Set base fee
                                                  // Check tiers from highest to lowest to find applicable discount
        // for i in 0..self.tiers.len() {
        //     if user_points >= *self.tiers.get(i).unwrap().threshold {
        //         let discount = U256::from(*self.tiers.get(i).unwrap().discount);
        //         let discounted_amount = (base_fee * discount) / U256::from(10000);
        //         return Ok(base_fee - discounted_amount);
        //     }
        // }

        // No tier matched, return base fee
        Ok(base_fee)
    }

    fn calculate_points_earned(
        &self,
        zero_for_one: bool,
        amount_specified: I256,
        delta_amount_0: I256,
    ) -> Result<U256, Error> {
        if zero_for_one {
            if amount_specified < I256::ZERO {
                return Ok(amount_specified.unsigned_abs());
            } else {
                return Ok(delta_amount_0.unsigned_abs());
            }
        } else {
            if amount_specified > I256::ZERO {
                return Ok(amount_specified.unsigned_abs());
            } else {
                return Ok(delta_amount_0.unsigned_abs());
            }
        }
    }
}


/// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, uint};

    const CURRENCY_ETH: Address = address!("0000000000000000000000000000000000000000");
    const CURRENCY_1: Address = address!("A11CEacF9aa32246d767FCCD72e02d6bCbcC375d");
    const CURRENCY_2: Address = address!("B0B0cB49ec2e96DF5F5fFB081acaE66A2cBBc2e2");

    #[test]
    fn sample_test() {
        assert_eq!(4, 2 + 2);
    }

    #[motsu::test]
    fn calculates_points_calculation(contract: FeeLogic) {

        let swap_amount = 100.try_into().unwrap();

        // Test baseline values
        let user = address!("199d51a2Be04C65f325908911430E6FF79a15ce3");
        let user_points = contract.get_user_points(user, CURRENCY_1).expect("should get user points for currency 1");
        assert_eq!(user_points, U256::ZERO);

        // // Update points
        contract.update_points(user, true, swap_amount, swap_amount, CURRENCY_ETH, CURRENCY_1).expect("should update points");
        // Check new values, points should be equal to swap amount
        let user_points = contract.get_user_points(user, CURRENCY_1).expect("should get user points for currency 1");
        assert_eq!(user_points, swap_amount.unsigned_abs());

    }

    // #[motsu::test]
    // fn calculates_amount_out(contract: UniswapCurve) {
    //     let amount_in = uint!(2_U256);
    //     let expected_amount_out = amount_in; // 1:1 swap
    //     let amount_out = contract
    //         .calculate_amount_out(amount_in, CURRENCY_1, CURRENCY_2, true)
    //         .expect("should calculate `amount_out`");
    //     assert_eq!(expected_amount_out, amount_out);
    // }

    // #[motsu::test]
    // fn returns_amount_in_for_exact_output(contract: UniswapCurve) {
    //     let amount_out = uint!(1_U256);
    //     let expected_amount_in = amount_out; // 1:1 swap
    //     let amount_in = contract
    //         .get_amount_in_for_exact_output(amount_out, CURRENCY_1, CURRENCY_2, true)
    //         .expect("should calculate `amount_in`");
    //     assert_eq!(expected_amount_in, amount_in);
    // }

    // #[motsu::test]
    // fn returns_amount_out_from_exact_input(contract: UniswapCurve) {
    //     let amount_in = uint!(2_U256);
    //     let expected_amount_out = amount_in; // 1:1 swap
    //     let amount_out = contract
    //         .get_amount_out_from_exact_input(amount_in, CURRENCY_1, CURRENCY_2, true)
    //         .expect("should calculate `amount_out`");
    //     assert_eq!(expected_amount_out, amount_out);
    // }
}