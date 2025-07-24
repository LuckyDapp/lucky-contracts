import {serve} from "bun";
import {TappdClient} from "@phala/dstack-sdk";
import {Keyring} from "@polkadot/keyring";
import type {KeyringPair} from "@polkadot/keyring/types";
import cron, {type ScheduledTask} from "node-cron";
import {type ContractConfig, type RegistrationContractId} from "./types.ts";
import {toHex} from "viem";
import {LottoWorker} from "./worker.ts";
import {computePrivateKey} from "@guigou/util-crypto";

const port = process.env.PORT || 3000;
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

function getRaffleManagerConfig() : ContractConfig {

  const address = process.env.MANAGER_ADDRESS;
  const rpc = process.env.MANAGER_RPC;
  const attestorKey = process.env.ATTESTOR_PK;

  if (!address){
    throw new Error("Manager address is missing!");
  }
  if (!rpc){
    throw new Error("Manager rpc is missing!");
  }
  if (!attestorKey){
    throw new Error("Manager attestor key is missing!");
  }
  return {
    address,
    rpc,
    attestorKey,
    senderKey: undefined,
  };
}

function getRaffleRegistrationConfigs() : Map<RegistrationContractId, ContractConfig> {

  let raffleRegistrationConfigs: Map<RegistrationContractId, ContractConfig> = new Map();
  const registration1Id = process.env.REGISTRATION_1_ID;
  const registration1rpc = process.env.REGISTRATION_1_RPC;
  const registration1address = process.env.REGISTRATION_1_ADDRESS;

  const attestorKey = process.env.ATTESTOR_PK;

  if (!registration1Id || !registration1rpc || !registration1address || !attestorKey){
    throw new Error("The config for the registration 1 is missing!");
  }
  raffleRegistrationConfigs.set(BigInt(registration1Id),
      {
        address: registration1address,
        rpc: registration1rpc,
        attestorKey,
        senderKey: undefined,
      }
  );

  const registration2Id = process.env.REGISTRATION_2_ID;
  const registration2rpc = process.env.REGISTRATION_2_RPC;
  const registration2address = process.env.REGISTRATION_2_ADDRESS;

  if (!registration2Id || !registration2rpc || !registration2address || !attestorKey){
    throw new Error("The config for the registration 2 is missing!");
  }
  raffleRegistrationConfigs.set(BigInt(registration2Id),
      {
        address: registration2address,
        rpc: registration2rpc,
        attestorKey,
        senderKey: undefined,
      }
  );

  return raffleRegistrationConfigs;
}


let worker : LottoWorker ;

function getOrCreateWorker() : LottoWorker {

  if (!worker) {
    const indexerUrl = process.env.INDEXER_URL;
    if (!indexerUrl) {
      throw new Error("Indexer url is missing!");
    }
    worker = new LottoWorker(
        getRaffleManagerConfig(),
        getRaffleRegistrationConfigs(),
        indexerUrl,
    )
  }
  return worker;
}



function getOrCreateTask() : ScheduledTask {

  if (!scheduledTask){
    // Every 15 minutes
    scheduledTask = cron.schedule('*/15 * * * *',
        async () => {
          try {
            const worker = getOrCreateWorker();
            await worker.pollMessages();
            await worker.closeRegistrationsIfNecessary();
            await worker.pollMessages();
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
        "<li><a href='/start'>/manager/start</a>: Start a scheduled task, running every 5 minutes, to orchestrate the contracts.</li>" +
        "<li><a href='/stop'>/manager/stop</a>: Stop the scheduled task.</li>" +
        "<li><a href='/execute'>/manager/execute</a>: Force the schedulled task.</li>" +
        "<li><a href='/config'>/config</a>: Display the config.</li>" +
        "<li><a href='/manager/status'>/manager/status</a>: Query and display the manager status.</li>" +
        "<li><a href='/manager/draw-number'>/manager/draw-number</a>: Query and display the manager draw number.</li>" +
        "<li><a href='/registration/:id/status'>/registration/:id/status</a>: Query and display the status for the given registration contract.</li>" +
        "<li><a href='/regisration/:id/draw-number'>/regisration/:id/draw-number</a>: Query and display the draw number for the given registration contract.</li>" +
        "<li><a href='/worker/account'>/worker/account</a>: Using the `deriveKey` API to generate a deterministic wallet for Polkadot, a.k.a. a wallet held by the TEE instance.</li>" +
        "<li><a href='/worker/tdx-quote'>/worker/tdx-quote</a>: The `reportdata` is the worker public key and generates the quote for attestation report via `tdxQuote` API.</li>" +
        "<li><a href='/worker/tdx-quote-raw'>/worker/tdx-quote-raw</a>: The `reportdata` is the worker public key and generates the quote for attestation report. The difference from `/tdx_quote` is that you can see the raw text `Price Feed Oracle` in [Attestation Explorer](https://proof.t16z.com/).</li>" +
        "<li><a href='/worker/info'>/worker/info</a>: Returns the TCB Info of the hosted CVM." +
        "" +
        "</li></ul></div>"
    ),

    "/manager/status": async (req) => {
      const worker = getOrCreateWorker();
      const status = await worker.getStatus();
      return new Response(JSON.stringify({
        status
      }));
    },

    "/manager/draw-number": async (req) => {
      const worker = getOrCreateWorker();
      const drawNumber = await worker.getDrawNumber();
      return new Response(JSON.stringify({
        drawNumber
      }));
    },

    "/start": async (req) => {
      const task = startScheduledTask();
      return new Response(JSON.stringify({task}));
    },

    "/stop": async (req) => {
      const task = stopScheduledTask();
      return new Response(JSON.stringify({task}));
    },

    "/execute": async (req) => {
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


