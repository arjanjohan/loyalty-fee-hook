# Uniswap Loyalty Points Fee Hook

This is an Arbitrum Stylus smart contract that is used by [LoyaltyPointsFeeHook.sol in this repo](https://github.com/arjanjohan/loyalty-points-fee-hook).

## Getting started

First start the Nitro testnode.

```shell
./scripts/nitro-testnode.sh -d -i
```

If you need to have some testnet tokens, you can use this script
```shell
./nitro-testnode/test-node.bash script send-l2 --to address_<address> --ethamount <amount>
```

With the following command you can deploy it to an Arbitrum chain

```shell
cargo stylus deploy --private-key $PRIVATE_KEY -e $RPC_URL --no-verify
```

For example
```shell
cargo stylus deploy --private-key=ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 -e=http://localhost:8547 --no-verify
```

## Tests

For unit testing, this example integrates the [motsu](https://github.com/OpenZeppelin/rust-contracts-stylus/tree/main/lib/motsu) library from OpenZeppelin. To run unit tests, you can simply use

```shell
cargo test --locked --features std --lib
```

Alternatively, you can use the bash script available [test-unit.sh](/scripts/test-unit.sh).

## Exporting Solidity ABI Interface

To export the Solidity ABI interface run the following command

```shell
cargo stylus export-abi
```

## Solidity Interface

This is the current Solidity ABI Interface for the contract

TODO.

<!-- ```solidity
interface IUniswapCurve {
    function getAmountInForExactOutput(uint256 amount_out, address input, address output, bool zero_for_one) external returns (uint256);

    function getAmountOutFromExactInput(uint256 amount_in, address input, address output, bool zero_for_one) external returns (uint256);

    error CurveCustomError();
}
``` -->
