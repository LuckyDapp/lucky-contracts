import {type AccountId, type ContractConfig, type Era, type Participant} from "./types.ts";
import {type HexString, Option} from "@guigou/sc-rollup-core";
import {hexAddPrefix, hexToU8a} from "@polkadot/util";
import {decodeAddress, encodeAddress} from "@polkadot/keyring";
import {InkClient} from "@guigou/sc-rollup-ink-v5";
import {
    accountIdsCodec,
    eraCodec,
    LAST_WINNER,
    NB_WINNERS,
    NEXT_ERA,
    type RaffleResponseMessage,
    raffleResponseMessageCodec
} from "./wasm_codec.ts";
import {Bytes} from "scale-ts";
import {Indexer} from "./indexer.ts";
import {Vrf} from "@guigou/util-crypto";

const MAX_ERA = 999999999;

export class RaffleConsumerContract {
    private readonly indexer;
    private readonly client: InkClient<Uint8Array, RaffleResponseMessage>;
    private readonly vrf : Vrf;

    constructor(config: ContractConfig | null, indexer: Indexer, vrf: Vrf) {


        if (!config) throw new Error('WasmContractNotConfigured');

        this.client = new InkClient<Uint8Array, RaffleResponseMessage>(
            config.rpc,
            config.address,
            hexAddPrefix(config.attestorKey),
            config.senderKey ? hexAddPrefix(config.senderKey) : undefined,
            Bytes(),
            raffleResponseMessageCodec
        );
        this.indexer = indexer;
        this.vrf = vrf;

    }

    async getNextEra(): Promise<Option<Era>> {
        try {
            return await this.client.getNumber(NEXT_ERA, 'u32');
        } catch (err) {
            console.error('Next era unknown in kv store');
            throw new Error('NextEraUnknown');
        }
    }

    async getNbWinners(): Promise<Option<number>> {
        try {
            return await this.client.getNumber(NB_WINNERS, 'u16');
        } catch (err) {
            console.error('Nb winners unknown in kv store');
            throw new Error('NbWinnersUnknown');
        }
    }

    async getLastWinners(): Promise<Option<AccountId[]>> {
        try {
            const bytes = await this.client.getBytes(LAST_WINNER);
            return bytes.map(accountIdsCodec.dec);
        } catch (err) {
            console.error('Last winners unknown in kv store');
            throw new Error('Last winnersUnknown');
        }
    }


    async runRaffle(targetEra: Era) {

        await this.client.startSession();

        const oEra = await this.getNextEra();
        if (!oEra){
            throw new Error('Era is not set');
        }
        let era = oEra.orElse(MAX_ERA);

        const oNbWinners = await this.getNbWinners();
        const nbWinners = oNbWinners.valueOf();
        if (!nbWinners){
            throw new Error('nbWinners is not set');
        }

        while (era < targetEra){
            console.log("Run raffle for era %s", era);
            const tx = await this.runRaffleForEra(era, nbWinners);
            console.log("Submit transaction : " + tx);
            era = (await this.getNextEra()).orElse(MAX_ERA);
        }
    }

    private async runRaffleForEra(era: Era, nbWinners: number): Promise<Option<HexString>> {

        const eraInfo = await this.indexer.getEraInfo(era);

        if (eraInfo.subPeriod.toUpperCase() == 'VOTING') {
            console.log(`Voting subPeriod for era: ${era} => skip the raffle`);

            const action = {
                era,
                skipped: true,
                rewards: BigInt(0),
                winners: [],
            };
            this.client.addAction(action);

        } else {
            console.log(`BuildAndEarn subPeriod for era: ${era} => run raffle`);


            const rewards = await this.indexer.getRewards(era);
            console.log(`Total rewards for this era: ${rewards}`);

            let participants = await this.indexer.getParticipants(eraInfo.period, era);
            console.log(`Nb of participants: ${participants.length}`);

            const oExcluded = await this.getLastWinners();
            const excluded = oExcluded.valueOf();
            if (!excluded){
                throw new Error('Last winners are not set');
            }
            const participantExcluded = convertAddressesToString(excluded);
            console.log(`Exclude these participants: ${participantExcluded}`);
            participants = excludeParticipants(participants, participantExcluded);

            let winners: string[] = [];
            for (let i = 0; i < nbWinners; i++) {
                const winner = this.selectWinner(era, participants);
                winners.push(winner);
                participants = excludeParticipants(participants, [winner]);
            }

            console.log(`Winners: ${winners}`);

            const action = {
                era,
                skipped: false,
                rewards,
                winners : convertAddressesFromString(winners),
            };

            this.client.addAction(action);
        }
        return this.client.commit();
    }


    private selectWinner(era: Era, participants: Participant[]): string {

        let totalTickets = 0;

        for (let i = 0; i < participants.length; i++) {
            totalTickets += participants[i].nbTickets;
        }

        if (totalTickets == 0) {
            throw new Error("NoMoreParticipant");
        }

        console.log(`Total tickets : ${totalTickets}`);
        // build the salt used by the vrf
        const vrfSalt = eraCodec.enc(era)
        // draw the number
        const winningTicket = this.vrf.getRandomNumber(vrfSalt, 0, totalTickets);

        console.log(`Winning ticket : ${winningTicket}`);
        let currentTicket = 0;
        for (let i = 0; i < participants.length; i++) {
            currentTicket += participants[i].nbTickets;
            if (currentTicket >= winningTicket) {
                return participants[i].address;
            }
        }
        throw new Error("NoWinnerFound");
    }
}

function excludeParticipants(participants: Participant[], excluded: string[]): Participant[] {
    if (excluded == undefined) {
        return participants;
    }
    return participants.filter((p, i) => !excluded.includes(p.address));
}

function convertAddressesToString(addresses: AccountId[]) : string[] {
    return addresses.map(convertAddressToString);
}

function convertAddressToString(address: AccountId) : string {
    return encodeAddress(address, 5);
    /*
    let address_hex: [u8; 32] = scale::Encode::encode(&address)
        .try_into()
        .expect("incorrect length");
    AccountId32::from(address_hex)
        .to_ss58check_with_version(Ss58AddressFormatRegistry::AstarAccount.into())
     */
}

function convertAddressesFromString(addresses: string[]) : AccountId[] {
    return addresses.map(convertAddressFromString);
}

function convertAddressFromString(address: string) : AccountId {
    return decodeAddress(address, false, 5);
}