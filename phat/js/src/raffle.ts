// *** YOU ARE LIMITED TO THE FOLLOWING IMPORTS TO BUILD YOUR PHAT CONTRACT     ***
// *** ADDING ANY IMPORTS WILL RESULT IN ERRORS & UPLOADING YOUR CODE TO PHALA  ***
// *** NETWORK WILL FAIL. IF YOU WANT TO KNOW MORE, JOIN OUR DISCORD TO SPEAK   ***
// *** WITH THE PHALA TEAM AT https://discord.gg/5HfmWQNX THANK YOU             ***
import "@phala/pink-env";
import {
    createStructDecoder,
    createStructEncoder,
    createVecDecoder,
    createVecEncoder,
    decodeStr,
    decodeU16,
    decodeU32,
    encodeBool,
    encodeStr,
    encodeU128,
    encodeU32,
    WalkerImpl,
} from "@scale-codec/core";

type HexString = `0x${string}`

type Input = {
    era: number,
    nbWinners: number,
    excluded: string[],
}

const decodeInput = createStructDecoder<Input>([
    ['era', decodeU32],
    ['nbWinners', decodeU16],
    ['excluded', createVecDecoder(decodeStr)],
]);


type Output = {
    era: number,
    skipped: boolean,
    rewards: bigint,
    winners: string[],
}

const encodeOutput = createStructEncoder<Output>([
    ['era', encodeU32],
    ['skipped', encodeBool],
    ['rewards', encodeU128],
    ['winners', createVecEncoder(encodeStr)],
]);

enum Error {
    FailedToDecodeInput = "FailedToDecodeInput",
    FailedToFetchReward = "FailedToFetchReward",
    FailedToDecodeReward = "FailedToDecodeReward",
    NoReward = "NoReward",
    FailedToFetchParticipant = "FailedToFetchParticipant",
    FailedToDecodeParticipant = "FailedToDecodeParticipant",
    NoMoreParticipant = "NoMoreParticipant",
    NoWinnerFound = "NoWinnerFound",
    NoBlockNumber = "NoBlockNumber",
    NoPeriod = "NoPeriod"
}

function isHexString(str: string): boolean {
    const regex = /^0x[0-9a-f]+$/;
    return regex.test(str.toLowerCase());
}

function stringToHex(str: string): string {
    var hex = "";
    for (var i = 0; i < str.length; i++) {
        hex += str.charCodeAt(i).toString(16);
    }
    return "0x" + hex;
}


export type EraInfo = {
    era: number;
    period: number;
    subPeriod: string;
}

function getEraInfo(graphApi: string, era: number): EraInfo {
    const headers = {
        "Content-Type": "application/json",
        "User-Agent": "phat-contract",
    };

    const query1 = JSON.stringify({
        query: `query {dAppStakingEras(filter: {era: {equalTo: \"${era}\"}}){nodes{ era, blockNumber}}}`
    });

    const body1 = stringToHex(query1);
    //
    // In Phat Function runtime, we not support async/await, you need use `pink.batchHttpRequest` to
    // send http request. The function will return an array of response.
    //
    const response1 = pink.batchHttpRequest(
        [
            {
                url: graphApi,
                method: "POST",
                headers,
                body: body1,
                returnTextBody: true,
            },
        ],
        10000
    )[0];

    if (response1.statusCode !== 200) {
        console.log(
            `Fail to read Graph api for rewards with status code: ${response1.statusCode}, error: ${
                response1.error || response1.body
            }}`
        );
        throw Error.FailedToFetchReward;
    }
    const respBody1 = response1.body;
    if (typeof respBody1 !== "string") {
        throw Error.FailedToDecodeReward;
    }

    const node1 = JSON.parse(respBody1).data.dAppStakingEras.nodes[0];

    if (node1 == undefined || node1.blockNumber == undefined || node1.blockNumber == 0) {
        console.log(`No Block Number: ${node1}`);
        throw Error.NoBlockNumber;
    }

    const blockNumber = node1.blockNumber;

    console.log(`Block Number for era ${node1.era}: ${blockNumber}`);

    const query2 = JSON.stringify({
        query: `query {dAppSubPeriods(filter: {blockNumber: {lessThanOrEqualTo: \"${blockNumber}\"}}, first: 1, orderBy: BLOCK_NUMBER_DESC){nodes{ period, subPeriod, blockNumber}}}`
    });

    const body2 = stringToHex(query2);
    //
    // In Phat Function runtime, we not support async/await, you need use `pink.batchHttpRequest` to
    // send http request. The function will return an array of response.
    //
    const response2 = pink.batchHttpRequest(
        [
            {
                url: graphApi,
                method: "POST",
                headers,
                body: body2,
                returnTextBody: true,
            },
        ],
        10000
    )[0];

    if (response2.statusCode !== 200) {
        console.log(
            `Fail to read Graph api for rewards with status code: ${response2.statusCode}, error: ${
                response2.error || response2.body
            }}`
        );
        throw Error.FailedToFetchReward;
    }
    const respBody2 = response2.body;
    if (typeof respBody2 !== "string") {
        throw Error.FailedToDecodeReward;
    }

    const node2 = JSON.parse(respBody2).data.dAppSubPeriods.nodes[0];

    if (node2 == undefined || node2.period == undefined || node2.period == 0) {
        console.log(`No Period: ${node2}`);
        throw Error.NoPeriod;
    }

    const period = node2.period;
    const subPeriod = node2.subPeriod;
    console.log(`Period ${period} and sub-period ${subPeriod} for era ${era}`);

    return {
        era: era.valueOf(),
        period,
        subPeriod
    };
}


function getRewards(graphApi: string, era: number): bigint {
    const headers = {
        "Content-Type": "application/json",
        "User-Agent": "phat-contract",
    };

    const query = JSON.stringify({
        query: `query {dAppRewards(filter: { era: { equalTo: \"${era}\"  } }) {nodes {amount, era}}}`
    });

    const body = stringToHex(query);
    //
    // In Phat Function runtime, we not support async/await, you need use `pink.batchHttpRequest` to
    // send http request. The function will return an array of response.
    //
    const response = pink.batchHttpRequest(
        [
            {
                url: graphApi,
                method: "POST",
                headers,
                body,
                returnTextBody: true,
            },
        ],
        10000
    )[0];

    if (response.statusCode !== 200) {
        console.log(
            `Fail to read Graph api for rewards with status code: ${response.statusCode}, error: ${
                response.error || response.body
            }}`
        );
        throw Error.FailedToFetchReward;
    }
    const respBody = response.body;
    if (typeof respBody !== "string") {
        throw Error.FailedToDecodeReward;
    }

    const node = JSON.parse(respBody).data.dAppRewards.nodes[0];

    if (node == undefined || node.amount == undefined || node.amount == 0) {
        console.log(`No rewards: ${node}`);
        throw Error.NoReward;
    }

    return node.amount;
}


export type Participant = {
    address: string;
    nbTickets: number;
}

interface GetParticipantsQueryResult {
    sum: { amount: string };
    keys: string[];
}

function getParticipants(graphApi: string, period: number, era: number): Participant[] {
    const headers = {
        "Content-Type": "application/json",
        "User-Agent": "phat-contract",
    };

    const query = JSON.stringify({
        query: `query { stakes(filter: {and: [ {period: {equalTo: \"${period}\"}}, {era: {lessThan: \"${era}\"}}]}) {groupedAggregates(groupBy: [ACCOUNT_ID], having: { sum: { amount: { notEqualTo: "0" }}}) { sum{amount}, keys }}}`
    });

    const body = stringToHex(query);
    //
    // In Phat Function runtime, we not support async/await, you need use `pink.batchHttpRequest` to
    // send http request. The function will return an array of response.
    //
    const response = pink.batchHttpRequest(
        [
            {
                url: graphApi,
                method: "POST",
                headers,
                body,
                returnTextBody: true,
            },
        ],
        10000
    )[0];

    if (response.statusCode !== 200) {
        console.log(
            `Fail to read Graph api with status code: ${response.statusCode}, error: ${
                response.error || response.body
            }}`
        );
        throw Error.FailedToFetchParticipant;
    }
    const respBody = response.body;
    if (typeof respBody !== "string") {
        throw Error.FailedToDecodeParticipant;
    }

    let participants: Participant[] = [];
    const participantsQueryResult: Array<GetParticipantsQueryResult> = JSON.parse(respBody).data.stakes.groupedAggregates;

    const ticketPrice = BigInt(500000000000000);

    for (let i = 0; i < participantsQueryResult.length; i++) {

        const address = participantsQueryResult[i].keys[0];
        const stake = participantsQueryResult[i].sum;
        const stakeBigInt = BigInt(parseFloat(stake.amount));
        const nbTickets = stakeBigInt / ticketPrice;

        participants.push({address, nbTickets: Number(nbTickets)});
    }

    console.log(`Number of participants: ${participants.length}`);
    return participants;

}

function excludeParticipants(participants: Participant[], excluded: string[]): Participant[] {
    if (excluded == undefined) {
        return participants;
    }
    return participants.filter((p, i) => !excluded.includes(p.address));
}

function selectWinner(participants: Participant[]): string {

    let totalTickets = 0;

    for (let i = 0; i < participants.length; i++) {
        totalTickets += participants[i].nbTickets;
    }

    if (totalTickets == 0) {
        throw Error.NoMoreParticipant;
    }

    const winnerTicket = Math.floor(Math.random() * totalTickets + 1);

    let currentTicket = 0;
    for (let i = 0; i < participants.length; i++) {
        currentTicket += participants[i].nbTickets;
        if (currentTicket >= winnerTicket) {
            return participants[i].address;
        }
    }
    throw Error.NoWinnerFound;
}

function parseInput(hexx: string): Input {
    let hex = hexx.toString();
    if (!isHexString(hex)) {
        throw Error.FailedToDecodeInput;
    }

    hex = hex.slice(2);

    let arr = new Array<number>();
    let i = 0;

    for (let c = 0; c < hex.length; c += 2) {
        arr[i++] = parseInt(hex.substring(c, c + 2), 16);
    }

    return WalkerImpl.decode(new Uint8Array(arr), decodeInput);
}

function formatOutput(output: Output): Uint8Array {
    return WalkerImpl.encode(output, encodeOutput);
}


//
// Here is what you need to implemented for Phat Function, you can customize your logic with
// JavaScript here.
//
// The function will be called with two parameters:
//
// - request: The raw payload from the contract call `request`.
//            In this example, it's a struct with the dAppId: { dappId }
// - settings: The custom settings you set with the `config_core` function of the Action Offchain Rollup Phat Contract.
//            In this example, it's just a simple text of the graph api url.
//
// Your returns value MUST be a Uint8Array, and it will send to your contract directly.
export default function main(request: HexString, settings: string): Uint8Array {

    console.log(`Request : ${request}`);
    const input = parseInput(request);
    const era = input.era;
    const nbWinners = input.nbWinners;
    const excluded = input.excluded;

    console.log(`Era ${era} - select ${nbWinners} address(es) excluding [${excluded}]`);
    console.log(`Settings: ${settings}`);

    const graphApi = settings;

    try {
        const eraInfo = getEraInfo(graphApi, era);

        if (eraInfo.subPeriod.toUpperCase() == 'VOTING') {
            const output: Output = {
                era: Number(era.toString()),
                skipped: true,
                rewards: BigInt(0),
                winners: [],
            }

            console.log(`Voting subPeriod for era: ${output.era} => skip the raffle`);

            return formatOutput(output);
        }

        const period = eraInfo.period;

        const rewards = getRewards(graphApi, era);
        let participants = getParticipants(graphApi, period, era);
        participants = excludeParticipants(participants, excluded);

        let winners: string[] = [];

        for (let i = 0; i < nbWinners; i++) {
            const winner = selectWinner(participants);
            winners.push(winner);
            participants = excludeParticipants(participants, [winner]);
        }

        const output: Output = {
            era: Number(era.toString()),
            skipped: false,
            rewards: BigInt(rewards.valueOf()),
            winners,
        }

        console.log(`winners: ${output.winners}`);
        console.log(`Rewards: ${output.rewards}`);

        return formatOutput(output);
    } catch (error) {
        console.log("error:", error);
        throw error;
    }
}
