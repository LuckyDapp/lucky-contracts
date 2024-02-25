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
  decodeU32,
  encodeStr, encodeU128,
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
  ['nbWinners', decodeU32],
  ['excluded', createVecDecoder(decodeStr)],
]);


type Output = {
  era: number,
  winners: string[],
  rewards: bigint,
}

const encodeOutput = createStructEncoder<Output>([
  ['era', encodeU32],
  ['winners', createVecEncoder(encodeStr)],
  ['rewards', encodeU128],
]);

enum Error {
  FailedToDecodeInput = "FailedToDecodeInput",
  FailedToFetchReward = "FailedToFetchReward",
  FailedToDecodeReward = "FailedToDecodeReward",
  NoReward = "NoReward",
  FailedToFetchParticipant = "FailedToFetchParticipant",
  FailedToDecodeParticipant = "FailedToDecodeParticipant",
  NoMoreParticipant = "NoMoreParticipant",
  NoWinnerFound = "NoWinnerFound"
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



function getRewards(graphApi: string, era: number): bigint {
  let headers = {
    "Content-Type": "application/json",
    "User-Agent": "phat-contract",
  };

  let query = JSON.stringify({
    query : `query {dAppRewards(filter: { era: { equalTo: \"${era}\"  } }) {nodes {amount, era}}}`
  });

  console.log("query: " + query);

  let body = stringToHex(query);
  //
  // In Phat Function runtime, we not support async/await, you need use `pink.batchHttpRequest` to
  // send http request. The function will return an array of response.
  //
  let response = pink.batchHttpRequest(
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
  let respBody = response.body;
  if (typeof respBody !== "string") {
    throw Error.FailedToDecodeReward;
  }

  console.log("respBody: " + respBody);
  const node = JSON.parse(respBody).data.dAppRewards.nodes[0];

  if (node == undefined || node.amount == undefined || node.amount == 0){
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
  sum: { amount : string };
  keys: string[];
}

function getParticipants(graphApi: string, period: number, era: number): Participant[] {
  let headers = {
    "Content-Type": "application/json",
    "User-Agent": "phat-contract",
  };

  let query = JSON.stringify({
    query : `query { stakes(filter: {and: [ {period: {equalTo: \"${period}\"}}, {era: {lessThan: \"${era}\"}}]}) {groupedAggregates(groupBy: [ACCOUNT_ID], having: { sum: { amount: { notEqualTo: "0" }}}) { sum{amount}, keys }}}`
  });

  console.log("query: " + query);

  let body = stringToHex(query);
  //
  // In Phat Function runtime, we not support async/await, you need use `pink.batchHttpRequest` to
  // send http request. The function will return an array of response.
  //
  let response = pink.batchHttpRequest(
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
  let respBody = response.body;
  if (typeof respBody !== "string") {
    throw Error.FailedToDecodeParticipant;
  }

  let participants: Participant[] = [];
  let participantsQueryResult : Array<GetParticipantsQueryResult> = JSON.parse(respBody).data.stakes.groupedAggregates;

  const ticketPrice = BigInt(500000000000000);

  for(let i=0; i<participantsQueryResult.length; i++){

    const address = participantsQueryResult[i].keys[0];
    const stake = participantsQueryResult[i].sum;
    const stakeBigInt = BigInt(parseFloat(stake.amount));
    const nbTickets = stakeBigInt / ticketPrice;

    participants.push({address, nbTickets: Number(nbTickets)});
  }

  console.log('Number of participants: %s', participants.length);
  return participants;

}


function excludeParticipants(participants: Participant[], excluded: string[]): Participant[] {
  if (excluded == undefined){
    return participants;
  }
  return participants.filter( (p,i) => ! excluded.includes(p.address) );
}

function selectWinner(participants: Participant[]): string {

  let totalTickets =  0;

  for(let i=0; i<participants.length; i++){
    totalTickets += participants[i].nbTickets;
  }

  if (totalTickets == 0){
    throw Error.NoMoreParticipant;
  }

  const winnerTicket = Math.floor(Math.random() * totalTickets + 1);

  let currentTicket =  0;
  for(let i=0; i<participants.length; i++){
    currentTicket += participants[i].nbTickets;
    if (currentTicket >= winnerTicket){
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

  let input = WalkerImpl.decode(new Uint8Array(arr), decodeInput);

  return input;
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
//export default function main(request: HexString, settings: string): Uint8Array {
export default function main(period: number, era: number, nbWinners: number, excluded: string[], settings: string): Uint8Array {

  //console.log(`handle req: ${request}`);
  console.log(`settings: ${settings}`);

  let input = parseInput(request);
  const graphApi = settings;
  const era = input.era;
  const nbWinners = input.nbWinners;
  const excluded = input.excluded;

  console.log(`Request received for period ${period} and era ${era}`);
  console.log(`Select ${nbWinners} address(es) excluding ${excluded}`);
  console.log(`Query endpoint ${graphApi}`);

  try {
    const rewards = getRewards(graphApi, era);
    let participants = getParticipants(graphApi, period, era);
    participants =  excludeParticipants(participants, excluded);

    let winners : string[] = [];

    for(let i=0; i<nbWinners; i++){
      const winner = selectWinner(participants);
      winners.push(winner);
      participants =  excludeParticipants(participants, [winner]);
    }

    const output: Output = {
      era : Number(era.toString()),
      winners,
      rewards : BigInt(rewards.valueOf()),
      //response_value: variant("Some", stats),
      //error: variant('None'),
    }

    console.log(`output - era: ${output.era}`);
    console.log(`output - winners: ${output.winners}`);
    console.log(`output - rewards: ${output.rewards}`);

    return formatOutput(output);
  } catch (error) {
    console.log("error:", error);
    throw error;
  }
}
