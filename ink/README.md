# Lucky Ink! Smart Contracts

## Smart contract `dapps_staking_developer`

This smart contract is registered as developer in the `dAppStaking` pallet and receives the rewards from dAppStaking.
The `raffle_consumer` contract is whitelisted to be able to withdraw these rewards and then transfer them into the `reward_manager` contract.

### Build the contract

```bash
cd contracts/dapps_staking_developer
cargo contract build
```

## Smart contract `reward_manager`

This smart contract manages the rewards that the lucky addresses can claim.
Only the `raffle_consumer` contract is granted to provide the list of winners. 

### Build the contract

```bash
cd contracts/reward_manager
cargo contract build
```

## Smart contract `raffle_consumer`

This smart contract :
 - consumes the output coming from the `raffle` phat contract that manages the raffle,
 - transfers funds from `dapps_staking_developer` contract to `reward_manager` contract,
 - provide the lucky address(es) to `reward_manager` contract.

Only the `raffle` phat contract is granted to provide the output of the raffle.

### Build the contract

```bash
cd contracts/raffle_consumer
cargo contract build
```

## Run e2e tests

Before you can run the test, you have to install a Substrate node with pallet-contracts. By default, e2e tests require that you install substrate-contracts-node. You do not need to run it in the background since the node is started for each test independently. To install the latest version:

```bash
cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git
```

If you want to run any other node with pallet-contracts you need to change CONTRACTS_NODE environment variable:

```bash
export CONTRACTS_NODE="YOUR_CONTRACTS_NODE_PATH"
```

And finally execute the following command to start e2e tests execution.

```bash
cd integration_tests
cargo test --features e2e-tests
```
