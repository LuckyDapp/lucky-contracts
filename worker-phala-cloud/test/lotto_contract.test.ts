import {expect, test} from "bun:test";
import type {ContractConfig} from "../src/types.ts";
import {RaffleConsumerContract} from "../src/lotto_contract.ts";

function getConfig() : ContractConfig {

    const address = process.env.CONTRACT_ADDRESS;
    const rpc = process.env.CONTRACT_RPC;
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

function getContract() : RaffleConsumerContract {

    const indexerUrl = process.env.INDEXER_URL;

    if (!indexerUrl){
        throw new Error("Indexer url is missing!");
    }
    return new RaffleConsumerContract(
        indexerUrl,
        getConfig(),
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

test("query currentEra", async () => {

    const contract = getContract();
    const currentEra = await contract.getCurrentEra();
    expect(currentEra.valueOf()).toBeGreaterThan(5899);
});



test("run raffle", async () => {

    const contract = getContract();
    await contract.runRaffle();
}, {timeout: 1200000});

