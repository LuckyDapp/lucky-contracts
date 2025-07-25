// TypeScript translation of the Rust `Indexer` struct and its methods

import axios from "axios";
import type {Era, EraInfo, Participant} from "./types.ts";

interface DAppStakingEraNode {
    era: number;
    blockNumber: number;
}

interface DAppStakingEraResponse {
    data: {
        dAppStakingEras: {
            nodes: DAppStakingEraNode[];
        };
    };
}

interface DAppSubPeriodNode {
    period: string;
    subPeriod: string;
    blockNumber: number;
}

interface DAppSubPeriodResponse {
    data: {
        dAppSubPeriods: {
            nodes: DAppSubPeriodNode[];
        };
    };
}

interface DAppRewardsNode {
    amount: bigint;
    era: number;
}

interface DAppRewardsResponse {
    data: {
        dAppRewards: {
            nodes: DAppRewardsNode[];
        };
    };
}

interface StakesGrouped {
    sum: { amount: string };
    keys: string[];
}

interface StakesResponse {
    data: {
        stakes: {
            groupedAggregates: StakesGrouped[];
        };
    };
}


export class Indexer {
    endpoint: string;

    constructor(url?: string) {
        if (!url) throw "IndexerNotConfigured";
        this.endpoint = url;
    }

    async getEraInfo(era: number): Promise<EraInfo> {
        if (!era) throw new Error("NoEra");

        const query1 = {
            query: `query {dAppStakingEras(filter: {era: {equalTo: \"${era}\"}}){nodes{ era, blockNumber}}}`
        };

        const response1 = await axios.post<DAppStakingEraResponse>(this.endpoint, query1, {
            headers: {
                "Content-Type": "application/json",
                Accept: "application/json",
            },
        }).catch(() => { throw new Error("HttpRequestFailed"); });

        const nodes1 = response1.data?.data?.dAppStakingEras?.nodes;
        if (!nodes1) throw new Error("InvalidResponseBody");
        if (nodes1.length != 1) throw new Error("No Block Number");

        const blockNumber = nodes1[0]?.blockNumber;

        if (blockNumber == undefined || blockNumber == 0) {
            throw new Error("No Block Number");
        }

        console.log(`Block Number for era ${era}: ${blockNumber}`);

        const query2 = {
            query: `query {dAppSubPeriods(filter: {blockNumber: {lessThanOrEqualTo: \"${blockNumber}\"}}, first: 1, orderBy: BLOCK_NUMBER_DESC){nodes{ period, subPeriod, blockNumber}}}`
        };

        const response2 = await axios.post<DAppSubPeriodResponse>(this.endpoint, query2, {
            headers: {
                "Content-Type": "application/json",
                Accept: "application/json",
            },
        }).catch(() => { throw new Error("HttpRequestFailed"); });

        const nodes2 = response2.data?.data.dAppSubPeriods.nodes;
        if (!nodes2) throw new Error("InvalidResponseBody");
        if (nodes2.length != 1) throw new Error("No Sub Period");

        const node2 = nodes2[0];

        if (node2 == undefined || node2.period == undefined) {
            throw new Error("No Sub Period");
        }

        const period = node2.period;
        const subPeriod = node2.subPeriod;
        console.log(`Period ${period} and sub-period ${subPeriod} for era ${era}`);

        return {
            era,
            period,
            subPeriod
        };
    }

    async getRewards(era: number): Promise<bigint> {
        if (!era) throw new Error("NoEra");

        const query = {
            query: `query {dAppRewards(filter: { era: { equalTo: \"${era}\"  } }) {nodes {amount, era}}}`
        };

        const response = await axios.post<DAppRewardsResponse>(this.endpoint, query, {
            headers: {
                "Content-Type": "application/json",
                Accept: "application/json",
            },
        }).catch(() => { throw new Error("FailedToFetchReward"); });


        const node = response.data?.data.dAppRewards.nodes[0];

        if (node == undefined || node.amount == undefined || node.amount == BigInt(0)) {
            console.log(`No rewards: ${node}`);
            throw new Error("NoReward");
        }

        return BigInt(node.amount);
    }


    async getParticipants(period: string, era: number): Promise<Participant[]> {
        if (!era) throw new Error("NoEra");

        const query = {
            query: `query { stakes(filter: {and: [ {period: {equalTo: \"${period}\"}}, {era: {lessThan: \"${era}\"}}]}) {groupedAggregates(groupBy: [ACCOUNT_ID], having: { sum: { amount: { notEqualTo: "0" }}}) { sum{amount}, keys }}}`
        };

        const response = await axios.post<StakesResponse>(this.endpoint, query, {
            headers: {
                "Content-Type": "application/json",
                Accept: "application/json",
            },
        }).catch(() => { throw new Error("FailedToFetchParticipant"); });

        let participants: Participant[] = [];
        const stakes = response.data?.data.stakes.groupedAggregates;
        // 500 ASTR
        const ticketPrice = BigInt('500000000000000000000');

        for (let i = 0; i < stakes.length; i++) {

            const address = stakes[i].keys[0];
            const stake = stakes[i].sum
            const stakeBigInt = BigInt(parseFloat(stake.amount));
            const nbTickets = stakeBigInt / ticketPrice;
            //console.log(`Stake: ${stakeBigInt}`);
            //console.log(`nbTickets: ${nbTickets}`);

            participants.push({address, nbTickets: Number(nbTickets)});
        }

        console.log(`Number of participants: ${participants.length}`);
        return participants;
    }


    async getLastEraReceivedReward(): Promise<Era> {

        const query = {
            query : 'query {dAppRewards(orderBy: ERA_DESC, first:1) {nodes {era}}}'
        };

        const response = await axios.post<DAppRewardsResponse>(this.endpoint, query, {
            headers: {
                "Content-Type": "application/json",
                Accept: "application/json",
            },
        }).catch(() => { throw new Error("FailedToFetchParticipant"); });

        const era = response.data?.data.dAppRewards.nodes[0].era;
        console.log('Last era when the dApp received the rewards: %s', era);
        return era;

    }

}
