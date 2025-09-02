export type Era = number;
export type Balance = bigint;
export type AccountId = Uint8Array; // 32 bytes

export enum Chain { Shibuya, Shiden, Astar}

export interface ContractConfig {
    rpc: string;
    address: string;
    attestorKey: string;
    senderKey?: string;
}

export interface DappStakingProxyConfig {
    rpc: string;
    address: string;
    privateKey: string;
}


export interface WorkerInfo {
    currentEra: Era;
    lastEraReceivedReward: Era;
    nextEra: Era;
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
