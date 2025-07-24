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
});

test("get rewards", async () => {
    const rewards = await indexer.getRewards(1110);
    expect(rewards).toBeGreaterThan(151932511267021804080n);
});

test("query salt 20", async () => {
    const participants = await indexer.getParticipants("7", 1110);
    expect(participants.length).toBe(132);
});
