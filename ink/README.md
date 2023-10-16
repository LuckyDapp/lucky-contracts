# Lucky-contracts
Smartcontracts to distribute the rewards received by the developer from dAppStaking.
Based on the configuration (ratioDistribition) 100%, 80%, ... of rewards will be distributed randomly to 1,2,3, ... lucky participant(s).


Structure of the project:
<pre>
 |-- contracts/
 |   |-- dapps_staking_developer/
 |       |-- lib.rs
 |   |-- lucky_raffle/
 |       |-- lib.rs
 |   |-- random_generator/
 |       |-- lib.rs
 |   |-- reward_manager/
 |       |-- lib.rs
 |-- logics/
 |   |-- traits/
 |       |-- participant_filter
 |           |-- filter_latest_winner.rs
 |       |-- reward
 |           |-- psp22_reward.rs
 |       |-- participant_manager.rs
 |       |-- raffle.rs    
 |       |-- random.rs
 |       |-- random_generators.rs
 |   |-- tests/
 |       |-- participant_manager.rs
 |       |-- psp22_reward.rs   
 |       |-- raffle.rs
 |       |-- random_generators.rs
 </pre>
 
## Smart contract 'dAppStaking Developer'

This smart contract will be registered as developer in the dAppStaking module and will receive rewards from dAppStaking.
The smart contract 'Raffle' will be whitelisted to be able to withdraw these rewards.

### Build the contract ###
```bash
cd contracts/dapps_staking_developer
cargo contract build
```

## Smart contract 'reward Manager'

This smart contract will manage rewards to distribute to the lucky addresses

### Build the contract ###
```bash
cd contracts/reward_manager
cargo contract build
```

## Smart contract 'random_generator'

This smart contract will act as an Oracle to provide the pseudo random number

### Build the contract ###
```bash
cd contracts/random_generator
cargo contract build
```


## Smart contract 'lucky Raffle'

This smart contract will :
 - randomly select address(es) in the list of participants
 - transfer the fund from 'dAppStacking developer' to 'reward Manager' contracts
 - set the lucky address(es) in the 'reward Manager' contract  

### Build the contract ###
```bash
cd contracts/lucky_raffle
cargo contract build
```


## Runs the tests

```bash
cargo test
```




