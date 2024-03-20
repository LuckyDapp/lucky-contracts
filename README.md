# Lucky Raffle

Lucky is a dApp built on top of dApp Staking. 
The dApp organizes a raffle per era to redistribute a share of the developer rewards to one or more lucky guys among stakers. 
It's a no-loss lottery!

## The raffle
The smart contract organizes a raffle among the addresses who staked on the dApp and distributes a share of the developer rewards to one lucky address.

It means that the user who stakes to the dApp Lucky will still receive the rewards from the dApp Staking in Astar, and moreover he will have a chance to win extra rewards with the raffles.

There is one raffle by era. The more you stake, the more chance you have to win a raffle.

When you stake 100 tokens, it means you have 100 tickets for the raffle. Total tickets are the sum of all staked tokens on the dApp at each raffle.

So more tickets means more chance to win!

To try to give everyone a chance and prevent a whale from getting all the rewards, a same address cannot win consecutively. It must wait 10 eras to participate in the lottery again. The number of eras to wait is configurable and can be adapted if necessary.

## Smart contracts

There are three ink! smart contracts deployed on Astar Network:
 - `dapp_staking_developer` : this contract receives the rewards from the `dAppStaking` pallet,
 - `reward_manager` : this contract contains the list of winners and the funds to be claimed. The users interact with this smart contract,
 - `raffle_consumer` : this smart contract consumes the output of the raffle managed by the `raffle` phat contract. This contract interacts with `dapp_staking_developer` contract to withdraw the required funds and with the `reward_manager` to provide the lucky addresses and the rewards. 

More information [here](ink/README.md]).

## Phat contracts

There are two phat contracts deployed on Phala Network:
- `dapp_staking` : this phat contract calls the `dAppStaking` pallet to claim the dApp rewards,
- `raffle` : this phat contract manages the raffle and sent the result to the `raffle_consumer` contract.

More information [here](phat/README.md]).