# Lucky Phat Contracts

## dAppStaking

The `dapp_staking` calls the `dAppStaking` pallet to claim the dApp rewards.

More information [here](contracts/dapp_staking)

## Raffle
 
Phat contract that manages the raffle.
The `raffle` phat contract:
- reads the data from `raffle_consumer` contract, 
- via a js script, queries the indexer to get the participants and runs the raffle,
- sends the output to `raffle_consumer` contract.

More information [here](contracts/raffle)

## Javascript/Typescript

The business logic for the raffle is coded in the [raffle](js/src/raffle.ts) typescript.
A javascript is generated from the typescript and provide to the `raffle` phat contract.
This way, we separated the business logic (written in typescript) and the transport/communication layer provided by the `raffle` phat contract.


