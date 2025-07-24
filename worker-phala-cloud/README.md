# Price Feed Worker deployed on Phala Cloud


This worker fetch the prices from CoinGecko and feed them into the wasm contract. 
This worker uses the [sc-rolluo-api](/sc-rollup-api) to connect to the wasm contract. It is developed with [Bun](https://bun.sh/) and deployed on [Phala Cloud](https://cloud.phala.network/).
This repo also includes a default Dockerfile and docker-compose.yml for deployment.

## Installation

```shell
bun i
cp env.example .env
```

We need to download the DStack simulator:

```shell
# Mac
wget https://github.com/Leechael/tappd-simulator/releases/download/v0.1.4/tappd-simulator-0.1.4-aarch64-apple-darwin.tgz
tar -xvf tappd-simulator-0.1.4-aarch64-apple-darwin.tgz
cd tappd-simulator-0.1.4-aarch64-apple-darwin
./tappd-simulator -l unix:/tmp/tappd.sock

# Linux
wget https://github.com/Leechael/tappd-simulator/releases/download/v0.1.4/tappd-simulator-0.1.4-x86_64-linux-musl.tgz
tar -xvf tappd-simulator-0.1.4-x86_64-linux-musl.tgz
cd tappd-simulator-0.1.4-x86_64-linux-musl
./tappd-simulator -l unix:/tmp/tappd.sock
```

Once the simulator is running, you need to open another terminal to start your Bun development server:

```shell
bun run dev
```

By default, the Bun development server will listen on port 3000. Open http://127.0.0.1:3000/fetch_prices in your browser to fetch the price from CoinGecko.

This repo also includes code snippets for the following common use cases:

- `/fetch-prices`: Using the `/fetch-prices` API to fetch the prices from CoinGecko.
- `/feed-prices/v5/start`: Using the `/feed-prices/v5/start` API to start a scheduled task, running every 5 minutes, to feed the prices into the ink! smart contract.
- `/feed-prices/v5/stop`: Using the `/feed-prices/v5/stop` API to stop the scheduled task.
- `/feed-prices/v5/execute`: Using the `/feed-prices/v5/execute` API to force to feed the prices into the ink! smart contract.
- `/feed-prices/v5/info`: Using the `/feed-prices/v5/info` API to display the information linked to the scheduled task.
- `/feed-prices/v5/attestor`: Using the `/feed-prices/v5/attestor` API to display the address used as attestor to feed the process. If the `INK_V5_ATTESTOR_PK` env key if not provided, the worker's address will be used.
- `/feed-prices/v5/sender`: Using the `/feed-prices/v5/sender` API to display the address used as sender in the context of meta-transaction. If the `INK_V5_SENDER_PK` env key if not provided, the meta tx is not enabled (ie the attestor is the sender).
- `/feed-prices/v6/start`: Using the `/feed-prices/v6/start` API to start a scheduled task, running every 5 minutes, to feed the prices into the ink! smart contract.
- `/feed-prices/v6/stop`: Using the `/feed-prices/v6/stop` API to stop the scheduled task.
- `/feed-prices/v6/execute`: Using the `/feed-prices/v6/execute` API to force to feed the prices into the ink! smart contract.
- `/feed-prices/v6/info`: Using the `/feed-prices/v6/info` API to display the information linked to the scheduled task.
- `/feed-prices/v6/attestor`: Using the `/feed-prices/v6/attestor` API to display the address used as attestor to feed the process. If the `INK_V6_ATTESTOR_PK` env key if not provided, the worker's address will be used.
- `/feed-prices/v6/sender`: Using the `/feed-prices/v6/sender` API to display the address used as sender in the context of meta-transaction. If the `INK_V6_SENDER_PK` env key if not provided, the meta tx is not enabled (ie the attestor is the sender).
- `/worker/account`: Using the `deriveKey` API to generate a deterministic wallet for Polkadot, a.k.a. a wallet held by the TEE instance.
- `/worker/tdx-quote`: The `reportdata` is `Price Feed Oracle` and generates the quote for attestation report via `tdxQuote` API.
- `/worker/tdx-quote-raw`: The `reportdata` is `Price Feed Oracle` and generates the quote for attestation report. The difference from `/tdx_quote` is that you can see the raw text `Price Feed Oracle` in [Attestation Explorer](https://proof.t16z.com/).
- `/worker/info`: Returns the TCB Info of the hosted CVM.


## Build

You need to build the image and push it to DockerHub for deployment. The following instructions are for publishing to a public registry via DockerHub:

```shell
sudo docker build . -t guigoudev/price-feed-oracle-phala-cloud
sudo docker push guigoudev/price-feed-oracle-phala-cloud
```

## Deploy

You can copy and paste the `docker-compose.yml` file from this repo to see the example up and running.