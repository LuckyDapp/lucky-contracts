import {test} from "bun:test";
import type {ContractConfig, DappStakingProxyConfig} from "../src/types.ts";
import {hexAddPrefix, hexToU8a} from "@polkadot/util";
import {LuckyWorker} from "../src/worker.ts";
import {clientContractAddress, developerContractAddress, indexerUrl, pk, rpc} from "./constants.ts";

function getRaffleContractConfig() : ContractConfig {

    if (!rpc){
        throw new Error("RPC is missing!");
    }
    if (!clientContractAddress){
        throw new Error("Raffle Consumer Contract address is missing!");
    }
    if (!pk){
        throw new Error("Attestor key is missing!");
    }
    return {
        address: clientContractAddress,
        rpc,
        attestorKey: pk,
        senderKey: undefined,
    };
}

function getDappStakingRaffleContractConfig() : DappStakingProxyConfig {

    if (!rpc){
        throw new Error("RPC is missing!");
    }
    if (!developerContractAddress){
        throw new Error("Developer Contract Address is missing!");
    }
    if (!pk){
        throw new Error("PK is missing!");
    }
    return {
        address : developerContractAddress,
        rpc,
        privateKey: hexAddPrefix(pk),
    };
}

function getWorker() : LuckyWorker {

    const raffleContractConfig = getRaffleContractConfig();
    const dappStakingRaffleContractConfig = getDappStakingRaffleContractConfig();

    if (!indexerUrl){
        throw new Error("Indexer URL is missing!");
    }
    if (!pk){
        throw new Error("VRF Seed is missing!");
    }

    return new LuckyWorker(
        indexerUrl,
        raffleContractConfig,
        dappStakingRaffleContractConfig,
        hexToU8a(pk),
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