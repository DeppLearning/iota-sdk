import type { IOutputResponse, ITransactionPayload } from '@iota/types';

/** Possible InclusionStates of transactions sent with the wallet */
export enum InclusionState {
    /** The transaction is pending */
    Pending = 'Pending',
    /** The transaction is confirmed */
    Confirmed = 'Confirmed',
    /** The transaction is conflicting */
    Conflicting = 'Conflicting',
    /** The transaction and its in- and outputs are pruned, so it's unknown if it got confirmed or was conflicting */
    UnknownPruned = 'UnknownPruned',
}

/** A Transaction with metadata */
export interface Transaction {
    /** The transaction payload */
    payload: ITransactionPayload;
    /** The block id in which the transaction payload was included */
    blockId?: string;
    /** The inclusion state of the transaction */
    inclusionState: InclusionState;
    /** The creation time */
    timestamp: string;
    /** The transaction id */
    transactionId: string;
    /** The network id in which the transaction was sent */
    networkId: string;
    /** If the transaction was created by the wallet or someone else */
    incoming: boolean;
    /** Note that can be set when sending a transaction and is only stored locally */
    note?: string;
    inputs: IOutputResponse[];
}
