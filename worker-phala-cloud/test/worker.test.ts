import {test} from "bun:test";
import type {ContractConfig, DappStakingProxyConfig} from "../src/types.ts";
import {hexAddPrefix, hexToU8a} from "@polkadot/util";
import {LuckyWorker} from "../src/worker.ts";

const rpc = process.env.RPC;

function getRaffleContractConfig() : ContractConfig {

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

function getWorker() : LuckyWorker {

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

    return new LuckyWorker(
        indexerUrl,
        raffleContractConfig,
        dappStakingRaffleContractConfig,
        hexToU8a(vrfSeed),
    )
}
/*

test("Claim All Eras", async () => {
    const worker = getWorker();
    await worker.claimAllEras();
}, {timeout: 1200000});

test("Run Raffles", async () => {
    const worker = getWorker();
    await worker.runRaffles();
}, {timeout: 1200000});

 */