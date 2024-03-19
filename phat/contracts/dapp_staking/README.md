# dApp Staking
Phat contract that claims the dAppStaking rewards.

## Build
```bash
cargo contract build
```

## Run Unit tests
```bash
cargo test  -- --nocapture
```

## Run Integration tests

Create the `.env` file with the following keys:
- `rpc`: The Substrate RPC for Phat Contract to send transaction. It must be a http endpoint.
- `pallet_id`: The `dAppStaking` pallet id for Phat Contract to send transaction.
- `call_id`: The `claimDappReward` call id for Phat Contract to send transaction.
- `smart_contract`: The smart contract given in parameter of teh method `dAppStaking.claimDappReward`.
- `sender_key`: The sr25519 private key you used to pay the transaction fees, with "0x".

```bash
cargo test  -- --ignored --test-threads=1
```

### Parallel in Integration Tests

The flag `--test-threads=1` is necessary because by default [Rust unit tests run in parallel](https://doc.rust-lang.org/book/ch11-02-running-tests.html).
There may have a few tests trying to send out transactions at the same time, resulting
conflicting nonce values.
The solution is to add `--test-threads=1`. So the unit test framework knows that you don't want
parallel execution.


