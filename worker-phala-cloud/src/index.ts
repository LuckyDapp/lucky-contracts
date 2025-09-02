import {serve} from "bun";
import {TappdClient} from "@phala/dstack-sdk";
import {Keyring} from "@polkadot/keyring";
import type {KeyringPair} from "@polkadot/keyring/types";
import cron, {type ScheduledTask} from "node-cron";
import {Chain, type ContractConfig, type DappStakingProxyConfig} from "./types.ts";
import {toHex} from "viem";
import {computePrivateKey} from "@guigou/util-crypto";
import {hexAddPrefix, hexToU8a} from "@polkadot/util";
import {LuckyWorker} from "./worker.ts";

const shibuyaClientRpc = process.env.SHIBUYA_RPC;
const shibuyaClientAddress = process.env.SHIBUYA_CLIENT_CONTRACT_ADDRESS;
const shibuyaDappStakingContractAddress = process.env.SHIBUYA_DAPP_STAKING_CONTRACT_ADDRESS;
const shibuyaWorkerPk = process.env.SHIBUYA_WORKER_PK;
const shibuyaIndexerUrl = process.env.SHIBUYA_INDEXER_URL;

const shidenClientRpc = process.env.SHIDEN_RPC;
const shidenClientAddress = process.env.SHIDEN_CLIENT_CONTRACT_ADDRESS;
const shidenDappStakingContractAddress = process.env.SHIDEN_DAPP_STAKING_CONTRACT_ADDRESS;
const shidenWorkerPk = process.env.SHIDEN_WORKER_PK;
const shidenIndexerUrl = process.env.SHIDEN_INDEXER_URL;

const astarClientRpc = process.env.ASTAR_RPC;
const astarClientAddress = process.env.ASTAR_CLIENT_CONTRACT_ADDRESS;
const astarDappStakingContractAddress = process.env.ASTAR_DAPP_STAKING_CONTRACT_ADDRESS;
const astarWorkerPk = process.env.ASTAR_WORKER_PK;
const astarIndexerUrl = process.env.ASTAR_INDEXER_URL;

const port = process.env.PORT || 3010;
console.log(`Listening on port ${port}`);


function displayChain(chain: Chain): string {
  if (chain == Chain.Astar ){
    return "Astar";
  } else if (chain == Chain.Shiden) {
    return "Shiden";
  } else if (chain == Chain.Shibuya) {
    return "Shibuya";
  }
  return "undefined";
}

function getChain(chain: string): Chain | undefined {
  if (chain.toUpperCase() == "ASTAR"){
    return Chain.Astar;
  }
  if (chain.toUpperCase() == "SHIDEN"){
    return Chain.Shiden;
  }
  if (chain.toUpperCase() == "SHIBUYA"){
    return Chain.Shibuya;
  }
  return undefined;
}

async function deriveKey(client: TappdClient, chain: Chain) : Promise<Uint8Array> {
  const deriveKeyResponse = await client.deriveKey(displayChain(chain));
  return computePrivateKey(deriveKeyResponse);
}

async function getSubstrateKeyringPair(client: TappdClient, chain: Chain) : Promise<KeyringPair> {
  const seed = await deriveKey(client, chain);
  return new Keyring({type: 'sr25519'}).addFromSeed(seed);
}

async function getRaffleContractConfig(client: TappdClient, chain: Chain) : Promise<ContractConfig> {

  let rpc, address, senderKey;
  if (chain == Chain.Astar) {
    rpc = astarClientRpc;
    address = astarClientAddress;
    senderKey = astarWorkerPk;
  } else if (chain == Chain.Shiden) {
    rpc = shidenClientRpc;
    address = shidenClientAddress;
    senderKey = shidenWorkerPk;
  } else {
    rpc = shibuyaClientRpc;
    address = shibuyaClientAddress;
    senderKey = shibuyaWorkerPk;
  }

  if (!rpc){
    throw new Error("RPC is missing!");
  }
  if (!address){
    throw new Error("Raffle Consumer Contract address is missing!");
  }

  const attestorKey = await deriveKey(client, chain);
  return {
    address,
    rpc,
    attestorKey : toHex(attestorKey),
    senderKey,
  };
}

function getDappStakingRaffleContractConfig(chain: Chain) : DappStakingProxyConfig {

  let rpc, address, workerKey;
  if (chain == Chain.Astar) {
    rpc = astarClientRpc;
    address = astarDappStakingContractAddress;
    workerKey = astarWorkerPk;
  } else if (chain == Chain.Shiden) {
    rpc = shidenClientRpc;
    address = shidenDappStakingContractAddress;
    workerKey = shidenWorkerPk;
  } else {
    rpc = shibuyaClientRpc;
    address = shibuyaDappStakingContractAddress;
    workerKey = shibuyaWorkerPk;
  }

  if (!rpc){
    throw new Error("RPC is missing!");
  }
  if (!address){
    throw new Error("Developer Contract Address is missing!");
  }
  if (!workerKey){
    throw new Error("Worker PK is missing!");
  }
  return {
    address,
    rpc,
    privateKey: hexAddPrefix(workerKey),
  };
}


function getIndexerUrl(chain: Chain) : string {

  let indexerUrl;
  if (chain == Chain.Astar) {
    indexerUrl = astarIndexerUrl;
  } else if (chain == Chain.Shiden) {
    indexerUrl = shidenIndexerUrl;
  } else {
    indexerUrl = shibuyaIndexerUrl;
  }
  if (!indexerUrl){
    throw new Error("Indexer URL is missing!");
  }
  return indexerUrl;
}

async function getVrfSeed(client: TappdClient, chain: Chain) : Promise<string> {
  const vrfSeed = await deriveKey(client, chain);
  return toHex(vrfSeed);
}

let astarWorker : LuckyWorker | undefined;
let shidenWorker : LuckyWorker | undefined;
let shibuyaWorker : LuckyWorker | undefined;

function getWorker(chain: Chain): LuckyWorker | undefined {
  if (chain == Chain.Astar) {
    return astarWorker;
  } else if (chain == Chain.Shiden) {
    return shidenWorker;
  } else {
    return shibuyaWorker;
  }
}

function setWorker(chain: Chain, worker: LuckyWorker | undefined) {
  if (chain == Chain.Astar ){
    astarWorker = worker;
  } else if (chain == Chain.Shiden) {
    shidenWorker = worker;
  } else {
    shibuyaWorker = worker;
  }
}

async function getOrCreateWorker(client: TappdClient, chain: Chain) : Promise<LuckyWorker> {
  let worker = getWorker(chain);
  if (worker == undefined) {
    const raffleContractConfig = await getRaffleContractConfig(client, chain);
    const dappStakingRaffleContractConfig = getDappStakingRaffleContractConfig(chain);
    const indexerUrl = getIndexerUrl(chain);
    const vrfSeed = await getVrfSeed(client, chain);
    
    worker = new LuckyWorker(
        indexerUrl,
        raffleContractConfig,
        dappStakingRaffleContractConfig,
        hexToU8a(vrfSeed),
    )
    setWorker(chain, worker);
  }
  return worker;
}

let astarScheduledTask : ScheduledTask | undefined;
let shidenScheduledTask : ScheduledTask | undefined;
let shibuyaScheduledTask : ScheduledTask | undefined;

function getScheduledTask(chain: Chain): ScheduledTask | undefined {
  if (chain == Chain.Astar) {
    return astarScheduledTask;
  } else if (chain == Chain.Shiden) {
    return shidenScheduledTask;
  } else {
    return shibuyaScheduledTask;
  }
}

function setScheduledTask(chain: Chain, task: ScheduledTask | undefined) {
  if (chain == Chain.Astar ){
    astarScheduledTask = task;
  } else if (chain == Chain.Shiden) {
    shidenScheduledTask = task;
  } else {
    shibuyaScheduledTask = task;
  }
}

async function getOrCreateTask(client: TappdClient, chain: Chain) : Promise<ScheduledTask> {
  let task = getScheduledTask(chain);
  if (task == undefined){
    // Every hour
    const worker = await getOrCreateWorker(client, chain);
    task = cron.schedule('0 * * * *',
        async () => {
          try {
            await worker.claimAllEras();
            await worker.runRaffles();
          } catch (e){
            console.error(e);
          }
        }, {noOverlap: true} );
    setScheduledTask(chain, task);
  }
  return task;
}

async function startScheduledTask(client: TappdClient, chain: Chain) {
  console.log('Start the scheduled task for ' + displayChain(chain));
  return getOrCreateTask(client, chain).then(task => task.start());
}

async function stopScheduledTask(client: TappdClient, chain: Chain) {
  console.log('Stop the scheduled task for ' + displayChain(chain));
  return getOrCreateTask(client, chain).then(task => task.stop());
}

async function executeScheduledTask(client: TappdClient, chain: Chain) {
  console.log('Execute the scheduled task for ' + displayChain(chain));
  return getOrCreateTask(client, chain).then(task => task.execute());
}

async function destroyWorkerAndScheduledTask(client: TappdClient, chain: Chain) {
  console.log('Destroy the scheduled task for ' + displayChain(chain));
  await getOrCreateTask(client, chain).then(task => task.destroy());
  setWorker(chain, undefined);
  setScheduledTask(chain, undefined);
}

serve({
  port,
  idleTimeout : 30,
  routes: {
    "/": new Response("" +
        "<h1>Lucky Worker</h1>" +

        "<h2>Astar Network</h2>" +
        "<div><ul>" +
        "<li><a href='/worker/astar/status'>/worker/astar/status</a>: Display the information comming from the dApp and the scheduled task status</li>" +
        "<li><a href='/worker/astar/start'>/worker/astar/start</a>: Start a scheduled task, running every hour, to claim the rewards and run the raffles.</li>" +
        "<li><a href='/worker/astar/stop'>/worker/astar/stop</a>: Stop the scheduled task.</li>" +
        "<li><a href='/worker/astar/execute'>/worker/astar/execute</a>: Force the scheduled task.</li>" +
        "<li><a href='/worker/astar/destroy'>/worker/astar/destroy</a>: Delete the worker to free memory</li>" +
        "<li><a href='/worker/astar/account'>/worker/astar/account</a>: Using the `deriveKey` API to generate a deterministic wallet for Polkadot, a.k.a. a wallet held by the TEE instance.</li>" +
        "<li><a href='/worker/astar/tdx-quote-raw'>/worker/astar/tdx-quote-raw</a>: The `reportdata` is the worker public key and generates the quote for attestation report. You can see the `worker public key` in <a href='https://proof.t16z.com' target='_blank'>Attestation Explorer</a>.</li>" +
        "</ul></div>" +

        "<h2>Shiden Network</h2>" +
        "<div><ul>" +
        "<li><a href='/worker/shiden/status'>/worker/shiden/status</a>: Display the information comming from the dApp and the scheduled task status</li>" +
        "<li><a href='/worker/shiden/start'>/worker/shiden/start</a>: Start a scheduled task, running every hour, to claim the rewards and run the raffles.</li>" +
        "<li><a href='/worker/shiden/stop'>/worker/shiden/stop</a>: Stop the scheduled task.</li>" +
        "<li><a href='/worker/shiden/execute'>/worker/shiden/execute</a>: Force the scheduled task.</li>" +
        "<li><a href='/worker/shiden/destroy'>/worker/shiden/destroy</a>: Delete the worker to free memory</li>" +
        "<li><a href='/worker/shiden/account'>/worker/shiden/account</a>: Using the `deriveKey` API to generate a deterministic wallet for Polkadot, a.k.a. a wallet held by the TEE instance.</li>" +
        "<li><a href='/worker/shiden/tdx-quote-raw'>/worker/shiden/tdx-quote-raw</a>: The `reportdata` is the worker public key and generates the quote for attestation report. You can see the `worker public key` in <a href='https://proof.t16z.com' target='_blank'>Attestation Explorer</a>.</li>" +
        "</ul></div>" +

        "<h2>Shibuya Network</h2>" +
        "<div><ul>" +
        "<li><a href='/worker/shibuya/status'>/worker/shibuya/status</a>: Display the information comming from the dApp and the scheduled task status</li>" +
        "<li><a href='/worker/shibuya/start'>/worker/shibuya/start</a>: Start a scheduled task, running every hour, to claim the rewards and run the raffles.</li>" +
        "<li><a href='/worker/shibuya/stop'>/worker/shibuya/stop</a>: Stop the scheduled task.</li>" +
        "<li><a href='/worker/shibuya/execute'>/worker/shibuya/execute</a>: Force the scheduled task.</li>" +
        "<li><a href='/worker/shibuya/destroy'>/worker/shibuya/destroy</a>: Delete the worker to free memory</li>" +
        "<li><a href='/worker/shibuya/account'>/worker/shibuya/account</a>: Using the `deriveKey` API to generate a deterministic wallet for Polkadot, a.k.a. a wallet held by the TEE instance.</li>" +
        "<li><a href='/worker/shibuya/tdx-quote-raw'>/worker/shibuya/tdx-quote-raw</a>: The `reportdata` is the worker public key and generates the quote for attestation report. You can see the `worker public key` in <a href='https://proof.t16z.com' target='_blank'>Attestation Explorer</a>.</li>" +
        "</ul></div>" +

        "<h2>TCB Info</h2>" +
        "<div><ul>" +
        "<li><a href='/worker/info'>/worker/info</a>: Returns the TCB Info of the hosted CVM." +
        "</ul></div>"
    ),

    "/worker/:chain/status": async (req) => {
      const chain = getChain(req.params.chain);
      if (chain == undefined){
        return new Response("Unknown chain!", {status: 503});
      }
      const client = new TappdClient();
      const worker = await getOrCreateWorker(client, chain).then(worker => worker.getInfos());
      const task = await getOrCreateTask(client, chain);
      return new Response(JSON.stringify({worker, task}));
    },

    "/worker/:chain/start": async (req) => {
      const chain = getChain(req.params.chain);
      if (chain == undefined){
        return new Response("Unknown chain!", {status: 503});
      }
      const client = new TappdClient();
      await startScheduledTask(client, chain);
      return new Response("The scheduled task has been started for " + displayChain(chain));
    },

    "/worker/:chain/stop": async (req) => {
      const chain = getChain(req.params.chain);
      if (chain == undefined){
        return new Response("Unknown chain!", {status: 503});
      }
      const client = new TappdClient();
      await stopScheduledTask(client, chain);
      return new Response("The scheduled task has been stopped for " + displayChain(chain));
    },

    "/worker/:chain/destroy": async (req) => {
      const chain = getChain(req.params.chain);
      if (chain == undefined){
        return new Response("Unknown chain!", {status: 503});
      }
      const client = new TappdClient();
      await destroyWorkerAndScheduledTask(client, chain);
      return new Response("The scheduled task has been destroyed for " + displayChain(chain));
    },

    "/worker/:chain/execute": async (req) => {
      const chain = getChain(req.params.chain);
      if (chain == undefined){
        return new Response("Unknown chain!", {status: 503});
      }
      const client = new TappdClient();
      executeScheduledTask(client, chain);
      return new Response("The scheduled task is executing for " + displayChain(chain));
    },

    "/worker/:chain/tdx-quote": async (req) => {
      const chain = getChain(req.params.chain);
      if (chain == undefined){
        return new Response("Unknown chain!", {status: 503});
      }
      const client = new TappdClient();
      const keypair = await getSubstrateKeyringPair(client, chain);
      const publicKey = toHex(keypair.publicKey).slice(2);
      const result = await client.tdxQuote(publicKey);
      return new Response(JSON.stringify(result));
    },

    "/worker/:chain/tdx-quote-raw": async (req) => {
      const chain = getChain(req.params.chain);
      if (chain == undefined){
        return new Response("Unknown chain!", {status: 503});
      }
      const client = new TappdClient();
      const keypair = await getSubstrateKeyringPair(client, chain);
      const publicKey = toHex(keypair.publicKey).slice(2);
      const result = await client.tdxQuote(publicKey, 'raw');
      return new Response(JSON.stringify(result));
    },

    "/worker/:chain/account": async (req) => {
      const chain = getChain(req.params.chain);
      if (chain == undefined){
        return new Response("Unknown chain!", {status: 503});
      }
      const client = new TappdClient();
      const keypair = await getSubstrateKeyringPair(client, chain);
      const ecdsaKeypair = new Keyring({type: 'ecdsa'}).addFromSeed(await deriveKey(client, chain));
      return new Response(JSON.stringify({
        sr25519Address: keypair.address,
        sr25519PublicKey: toHex(keypair.publicKey),
        ecdsaAddress: ecdsaKeypair.address,
        ecdsaPublicKey: toHex(ecdsaKeypair.publicKey),
      }));
    },

    "/worker/info": async () => {
      const client = new TappdClient();
      const result = await client.info();
      return new Response(JSON.stringify(result));
    },

  },
});


