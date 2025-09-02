import {expect, test} from "bun:test";
import {DappStakingProxy} from "../src/dapp_staking.ts";
import {hexAddPrefix} from "@polkadot/util";
import {developerContractAddress, pk, rpc} from "./constants.ts";

function getDappStakingProxy() : DappStakingProxy {

    if (!rpc){
        throw new Error("RPC is missing!");
    }
    if (!pk){
        throw new Error("PK is missing!");
    }
    if (!developerContractAddress){
        throw new Error("Developer Contract Address is missing!");
    }
    return new DappStakingProxy(
        {
            address: developerContractAddress,
            rpc,
            privateKey: hexAddPrefix(pk),
        }
    )
}


test("query current era", async () => {
    const proxy = getDappStakingProxy();
    const currentEra = await proxy.getCurrentEra();
    expect(currentEra).toBeNumber();
    console.log("Current era :" + currentEra);
});

/*
test("claim reward", async () => {
    const proxy = getDappStakingProxy();
    console.log("Claim Reward ... ");
    await proxy.claimReward(1189)
}, {timeout: 1200000});
*/