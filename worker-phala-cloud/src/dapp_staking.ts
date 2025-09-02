import {astar} from "@polkadot-api/descriptors";
import {createClient, type PolkadotClient} from "polkadot-api";
import {withPolkadotSdkCompat} from "polkadot-api/polkadot-sdk-compat";
import {getWsProvider} from "polkadot-api/ws-provider/web";
import type {SS58String} from "@polkadot-api/substrate-bindings";
import {getPolkadotSigner, type PolkadotSigner} from "polkadot-api/signer"
import {Keyring} from "@polkadot/keyring";
import {hexToU8a} from "@polkadot/util";
import type {DappStakingProxyConfig} from "./types.ts";

export class DappStakingProxy {
    private readonly polkadotClient : PolkadotClient;
    private readonly smartContractAddress : SS58String
    private readonly signer: PolkadotSigner;

    constructor(config: DappStakingProxyConfig) {

        this.smartContractAddress = config.address;
        this.polkadotClient = createClient(withPolkadotSdkCompat(getWsProvider(config.rpc)));
        const keypair = new Keyring({ type: "sr25519" }).addFromSeed(
            hexToU8a(config.privateKey),
        );
        this.signer = getPolkadotSigner(
            keypair.publicKey,
            "Sr25519",
            keypair.sign,
        )
    }


    async getCurrentEra() : Promise<number> {
        const currentEraInfo = await this.polkadotClient.getTypedApi(astar).query.DappStaking.CurrentEraInfo.getValue();
        return currentEraInfo.current_stake_amount.era;
    }

    async claimReward(
        era: number
    ) : Promise<void>{
        const result = await this.polkadotClient.getTypedApi(astar).tx.DappStaking.claim_dapp_reward(
            {
                era,
                smart_contract: {  type: "Wasm", value: this.smartContractAddress },
            }
        ).signAndSubmit(this.signer);
        if (!result.ok) {
            console.error("Error when claiming dapp reward");
            console.error(result);
            throw new Error("Error when claiming dapp reward : " + result);
        }
        console.log("claim dapp reward : " +  result.txHash);
    }

}