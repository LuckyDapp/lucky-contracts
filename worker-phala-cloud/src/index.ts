import {serve} from "bun";
import {TappdClient} from "@phala/dstack-sdk";
import {Keyring} from "@polkadot/keyring";
import type {KeyringPair} from "@polkadot/keyring/types";
import cron, {type ScheduledTask} from "node-cron";
import {type ContractConfig, type DappStakingProxyConfig} from "./types.ts";
import {toHex} from "viem";
import {computePrivateKey} from "@guigou/util-crypto";
import {hexAddPrefix, hexToU8a} from "@polkadot/util";
import {LuckyWorker} from "./worker.ts";

const port = process.env.PORT || 3100;
console.log(`Listening on port ${port}`);

let scheduledTask: ScheduledTask | undefined = undefined;

async function deriveKey(client: TappdClient) : Promise<Uint8Array> {
  const deriveKeyResponse = await client.deriveKey('polkadot');
  return computePrivateKey(deriveKeyResponse);
}

async function getSubstrateKeyringPair(client: TappdClient) : Promise<KeyringPair> {
  const seed = await deriveKey(client);
  return new Keyring({type: 'sr25519'}).addFromSeed(seed);
}



function getRaffleContractConfig() : ContractConfig {

  const rpc = process.env.RPC;
  const address = process.env.RAFFLE_CONTRACT_ADDRESS;
  const attestorKey = process.env.WORKER_PK;

  if (!rpc){
    throw new Error("RPC is missing!");
  }
  if (!address){
    throw new Error("Raffle Consumer Contract address is missing!");
  }
  if (!attestorKey){
    throw new Error("Attestor key is missing!");
  }
  return {
    address,
    rpc,
    attestorKey,
    senderKey: undefined,
  };
}

function getDappStakingRaffleContractConfig() : DappStakingProxyConfig {

  const rpc = process.env.RPC;
  const address = process.env.DAPP_STAKING_CONTRACT_ADDRESS;
  const pk = process.env.WORKER_PK;


  if (!rpc){
    throw new Error("RPC is missing!");
  }
  if (!address){
    throw new Error("Developer Contract Address is missing!");
  }
  if (!pk){
    throw new Error("PK is missing!");
  }
  return {
    address,
    rpc,
    privateKey: hexAddPrefix(pk),
  };
}

let worker : LuckyWorker ;

function getOrCreateWorker() : LuckyWorker {

  if (!worker) {
    const raffleContractConfig = getRaffleContractConfig();
    const dappStakingRaffleContractConfig = getDappStakingRaffleContractConfig();

    const indexerUrl = process.env.INDEXER_URL;
    if (!indexerUrl){
      throw new Error("Indexer URL is missing!");
    }
    const vrfSeed = process.env.WORKER_PK;
    if (!vrfSeed){
      throw new Error("VRF Seed is missing!");
    }
    worker = new LuckyWorker(
        indexerUrl,
        raffleContractConfig,
        dappStakingRaffleContractConfig,
        hexToU8a(vrfSeed),
    )
  }
  return worker;
}


function getOrCreateTask() : ScheduledTask {

  if (!scheduledTask){
    // Every hour
    scheduledTask = cron.schedule('0 * * * *',
        async () => {
          try {
            const worker = getOrCreateWorker();
            await worker.claimAllEras();
            await worker.runRaffles();
          } catch (e){
            console.error(e);
          }
        });
  }
  return scheduledTask;
}

function startScheduledTask() : ScheduledTask {
  console.log('Start the scheduled task');
  const task = getOrCreateTask();
  task.start();
  return task;
}

function stopScheduledTask() : ScheduledTask {
  console.log('Stop the scheduled task');
  const task = getOrCreateTask();
  task.stop();
  return task;
}

function executeScheduledTask() : ScheduledTask {
  console.log('Execute the scheduled task');
  const task = getOrCreateTask();
  task.execute();
  return task;
}

serve({
  port,
  idleTimeout : 30,
  routes: {
    "/": new Response("" +
        "<h1>Lucky Worker</h1>" +
        "<div><ul>" +
        "<li><a href='/lucky/infos'>/lucky/infos</a>: Display the information comming from the dApp.</li>" +
        "<li><a href='/worker/status'>/worker/status</a>: Display the scheduled task status</li>" +
        "<li><a href='/worker/start'>/worker/start</a>: Start a scheduled task, running every hour, to claim the rewards and run the raffles.</li>" +
        "<li><a href='/worker/stop'>/worker/stop</a>: Stop the scheduled task.</li>" +
        "<li><a href='/worker/execute'>/worker/execute</a>: Force the schedulled task.</li>" +
        "<li><a href='/worker/account'>/worker/account</a>: Using the `deriveKey` API to generate a deterministic wallet for Polkadot, a.k.a. a wallet held by the TEE instance.</li>" +
        "<li><a href='/worker/tdx-quote'>/worker/tdx-quote</a>: The `reportdata` is the worker public key and generates the quote for attestation report via `tdxQuote` API.</li>" +
        "<li><a href='/worker/tdx-quote-raw'>/worker/tdx-quote-raw</a>: The `reportdata` is the worker public key and generates the quote for attestation report. The difference from `/tdx_quote` is that you can see the raw text `Price Feed Oracle` in [Attestation Explorer](https://proof.t16z.com/).</li>" +
        "<li><a href='/worker/info'>/worker/info</a>: Returns the TCB Info of the hosted CVM." +
        "" +
        "</li></ul></div>"
    ),

    "/lucky/infos": async (req) => {
      const worker = getOrCreateWorker();
      const infos = await worker.getInfos();
      return new Response(JSON.stringify(infos));
    },

    "/worker/status": async (req) => {
      const task = getOrCreateTask();
      return new Response(JSON.stringify({task}));
    },

    "/worker/start": async (req) => {
      const task = startScheduledTask();
      return new Response(JSON.stringify({task}));
    },

    "/worker/stop": async (req) => {
      const task = stopScheduledTask();
      return new Response(JSON.stringify({task}));
    },

    "/worker/execute": async (req) => {
      const task = executeScheduledTask();
      return new Response(JSON.stringify({task}));
    },

    "/worker/info": async (req) => {
      const client = new TappdClient();
      const result = await client.info();
      return new Response(JSON.stringify(result));
    },

    "/worker/tdx-quote": async (req) => {
      const client = new TappdClient();
      const keypair = await getSubstrateKeyringPair(client);
      const publicKey = toHex(keypair.publicKey).slice(2);
      const result = await client.tdxQuote(publicKey);
      return new Response(JSON.stringify(result));
    },

    "/worker/tdx-quote-raw": async (req) => {
      const client = new TappdClient();
      const keypair = await getSubstrateKeyringPair(client);
      const publicKey = toHex(keypair.publicKey).slice(2);
      const result = await client.tdxQuote(publicKey, 'raw');
      return new Response(JSON.stringify(result));
    },

    "/worker/account": async (req) => {
      const client = new TappdClient();
      const keypair = await getSubstrateKeyringPair(client);
      const ecdsaKeypair = new Keyring({type: 'ecdsa'}).addFromSeed(await deriveKey(client));
      return new Response(JSON.stringify({
        sr25519Address: keypair.address,
        sr25519PublicKey: toHex(keypair.publicKey),
        ecdsaAddress: ecdsaKeypair.address,
        ecdsaPublicKey: toHex(ecdsaKeypair.publicKey),
      }));
    },

  },
});


