import {expect, test} from "bun:test";
import {Indexer} from "../src/indexer.ts";

const indexer = new Indexer("https://query.substrate.fi/lucky-subquery-shiden");

test("get era info", async () => {
    const eraInfo = await indexer.getEraInfo(1110);
    expect(eraInfo.era).toBeNumber();
    expect(eraInfo.era).toBe(1110);
    expect(eraInfo.period).toBeString();
    expect(eraInfo.period).toBe("7");
    expect(eraInfo.subPeriod).toBeString();
    expect(eraInfo.subPeriod).toBe("BuildAndEarn");


    for (let era = 1194 ; era > 1184; era--){
        console.log("info for " + era +  "  : " + await indexer.getEraInfo(era));
    }

});

test("get rewards", async () => {
    const rewards = await indexer.getRewards(1110);
    expect(rewards).toBeGreaterThan(151932511267021804080n);

    for (let era = 1194 ; era > 1184; era--){
        try {
            console.log("rewards for " + era + "  : " + await indexer.getRewards(era));
        } catch (e){
            if (e.message == "NoReward"){
                console.log("No reward for " + era);
            } else {
                throw e;
            }
        }
    }
});

test("query salt 20", async () => {
    const participants = await indexer.getParticipants("7", 1110);
    expect(participants.length).toBe(132);
});


test("query get Last Era Received Reward", async () => {
    const era = await indexer.getLastEraReceivedReward();
    console.log("Last Era Received Reward" + era);
    expect(era).toBeNumber();
    expect(era).toBeGreaterThan(1000);
});

