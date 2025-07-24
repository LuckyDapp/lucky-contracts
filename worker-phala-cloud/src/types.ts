export type Era = number;
export type Balance = bigint;
export type AccountId = Uint8Array; // 32 bytes

export interface ContractConfig {
    rpc: string;
    address: string;
    attestorKey: string;
    senderKey?: string;
}

export type EraInfo = {
    era: Era;
    period: string;
    subPeriod: string;
}

export type Participant = {
    address: string;
    nbTickets: number;
}
