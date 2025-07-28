import {type ContractConfig, type DappStakingProxyConfig, type WorkerInfo,} from './types';
import {Indexer} from "./indexer.ts";
import {Vrf} from "@guigou/util-crypto";
import {DappStakingProxy} from "./dapp_staking.ts";
import {RaffleConsumerContract} from "./lucky_raffle.ts";

export class LuckyWorker {

    private readonly dappStakingProxy: DappStakingProxy;
    private readonly indexer: Indexer;
    private readonly vrf: Vrf;
    private readonly raffleConsumerContract: RaffleConsumerContract;

    constructor(
        graphApi : string,
        raffleContractConfig: ContractConfig,
        dappStakingProxyConfig: DappStakingProxyConfig,
        vrfSeed: Uint8Array
    ) {
        this.indexer = new Indexer(graphApi);
        this.vrf = Vrf.getFromSeed(vrfSeed);
        this.raffleConsumerContract = new RaffleConsumerContract(raffleContractConfig, this.indexer, this.vrf);
        this.dappStakingProxy = new DappStakingProxy(dappStakingProxyConfig);
    }

    async getInfos() : Promise<WorkerInfo>{
        const currentEra = await this.dappStakingProxy.getCurrentEra();
        const lastEraReceivedReward = await this.indexer.getLastEraReceivedReward();
        const nextEra = await this.raffleConsumerContract.getNextEra();
        return {
            currentEra,
            lastEraReceivedReward,
            nextEra : nextEra.orElse(-1),
        }
    }

    async claimAllEras() : Promise<void>{
        const currentEra = await this.dappStakingProxy.getCurrentEra();
        const lastEraReceivedReward = await this.indexer.getLastEraReceivedReward();

        let era = +lastEraReceivedReward + 1;
        console.log("Claim all raffles - current era %s - era %s", currentEra, era);
        while (era < currentEra){
            const eraInfo = await this.indexer.getEraInfo(era);
            if (eraInfo.subPeriod.toUpperCase() != "VOTING") {
                console.log("Claim reward for era %s", era);
                await this.dappStakingProxy.claimReward(eraInfo.era);
            } else {
                console.log("No reward for era (voting sub-period)", era);
            }
            era++;
        }
    }

    async runRaffles() {
        const lastEraReceivedReward = await this.indexer.getLastEraReceivedReward();
        await this.raffleConsumerContract.runRaffle(lastEraReceivedReward);

    }

}
