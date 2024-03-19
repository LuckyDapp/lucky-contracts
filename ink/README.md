# Lucky-contracts
Smart contracts to distribute the rewards received by the developer from dAppStaking.
Based on the configuration (ratio distribution) 100%, 80%, ... of rewards will be distributed randomly to 1,2,3, ... lucky participant(s).
 
## Smart contract 'dAppStaking Developer'

This smart contract will be registered as developer in the dAppStaking module and will receive rewards from dAppStaking.
The smart contract 'Raffle Consumer' will be whitelisted to be able to withdraw these rewards.

### Build the contract ###
```bash
cd contracts/dapps_staking_developer
cargo contract build
```

## Smart contract 'Reward Manager'

This smart contract will manage rewards to distribute to the lucky addresses

### Build the contract ###
```bash
cd contracts/reward_manager
cargo contract build
```

## Smart contract 'Raffle Consumer'

This smart contract will :
 - consume data coming from the phat contract that manages the raffle.
 - transfer the fund from 'dAppStacking developer' to 'reward Manager' contracts
 - set the lucky address(es) in the 'Reward Manager' contract  

### Build the contract ###
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
cd contracts/raffle_consumer
cargo contract build
```






