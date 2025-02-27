// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { Wallet, initLogger } from '@iota/sdk';

// Run with command:
// yarn run-example ./how_tos/accounts_and_addresses/list_outputs.ts

// This example lists all outputs in the account
async function run() {
    initLogger();
    try {
        const wallet = new Wallet({
            storagePath: process.env.WALLET_DB_PATH,
        });

        const account = await wallet.getAccount(
            `${process.env.ACCOUNT_ALIAS_1}`,
        );

        await account.sync();

        const outputs = await account.outputs();

        console.log('Output ids:');
        for (const output of outputs) console.log(output.outputId);

        const unspentOutputs = await account.unspentOutputs();

        console.log('Unspent output ids:');
        for (const output of unspentOutputs) console.log(output.outputId);
    } catch (error) {
        console.error('Error: ', error);
    }
}

run().then(() => process.exit());
