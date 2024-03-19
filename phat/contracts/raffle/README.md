# Raffle
Phat contract that manages the raffle.
The `raffle` phat contract:
- reads the data from `raffle_consumer` ink! smart contract, 
- via a js script, queries the indexer and runs the raffle among the list of participants,
- sends the data into `raffle_consumer` ink! smart contract.

## Build ##
```bash
cargo contract build
```

## Run Unit tests

```bash
cargo test
```

## Run Integration tests
Unfortunately, the cross contract call doesn't work in a local environment. 
It means the JS contract used to manage the raffle can not been reached and the integration tests can not be run for the time being.