# Lucky Worker deployed on Phala Cloud


This worker claim the rewards from dAppStaking and run the raffle. 
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

By default, the Bun development server will listen on port 3000. Open http://127.0.0.1:3010/ in your browser to display teh menu.

## Build

You need to build the image and push it to DockerHub for deployment. The following instructions are for publishing to a public registry via DockerHub:

```shell
sudo docker build . -t guigoudev/lucky-worker-phala-cloud
sudo docker push guigoudev/lucky-worker-phala-cloud
```

## Deploy

You can copy and paste the `docker-compose.yml` file from this repo to see the example up and running.