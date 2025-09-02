import {type AccountId, type Balance, type Era} from './types';
import {bool, Bytes, type Codec, Struct, u128, u32, Vector} from "scale-ts";


// Constants
export const NEXT_ERA = '0xe6608356'; // assuming ink::selector_id!("NEXT_ERA")
export const NB_WINNERS = '0x021f707b'; // assuming ink::selector_id!("NB_WINNERS")
export const LAST_WINNER = '0x3d96da39'; // assuming ink::selector_id!("LAST_WINNER")


export const accountIdCodec : Codec<AccountId> = Bytes(32);
export const accountIdsCodec : Codec<AccountId[]> = Vector(accountIdCodec);

export const eraCodec : Codec<Era> = u32;

/*
    #[ink::scale_derive(Encode, Decode)]
    pub struct RaffleResponseMessage {
        pub era: u32,
        pub skipped: bool,
        pub rewards: Balance,
        pub winners: Vec<AccountId>,
    }
 */

export type RaffleResponseMessage = {
    era: Era,
    skipped: boolean,
    rewards: Balance,
    winners: AccountId[],
}

export const raffleResponseMessageCodec : Codec<RaffleResponseMessage> = Struct({
    era: eraCodec,
    skipped: bool,
    rewards: u128,
    winners: Vector(accountIdCodec)
});


