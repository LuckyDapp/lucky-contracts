import {expect, test} from "bun:test";
import type {ContractConfig} from "../src/types.ts";
import {RaffleConsumerContract} from "../src/lucky_raffle.ts";
import {Indexer} from "../src/indexer.ts";
import {Vrf} from "@guigou/util-crypto";
import {hexToU8a} from "@polkadot/util";
import {clientContractAddress, pk, rpc} from "./constants.ts";

function getConfig() : ContractConfig {


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

function getContract() : RaffleConsumerContract {

    const config = getConfig();

    const indexerUrl = process.env.SHIBUYA_INDEXER_URL;
    if (!indexerUrl){
        throw new Error("Indexer url is missing!");
    }
    const indexer = new Indexer(indexerUrl);
    const vrf = Vrf.getFromSeed(hexToU8a(config.attestorKey));

    return new RaffleConsumerContract(
        config,
        indexer,
        vrf,
    )
}


test("query data", async () => {

    const contract = getContract();
    const nextEra = await contract.getNextEra();
    expect(nextEra.isSome())
    expect(nextEra.valueOf()).toBeGreaterThan(1000);
    const nbWinners = await contract.getNbWinners();
    expect(nbWinners.isSome())
    expect(nbWinners.valueOf()).toBe(1);
    const lastWinners = await contract.getLastWinners();
    expect(lastWinners.isSome()).toBeTrue();
    expect(lastWinners.valueOf()?.length).toBeGreaterThan(0);
});

/*
test("run raffle", async () => {

    const contract = getContract();
    await contract.runRaffle(1185);
}, {timeout: 1200000});
 */
