// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import {
    Client,
    initLogger,
    ImmutableAliasAddressUnlockCondition,
    AliasAddress,
    SimpleTokenScheme,
} from '@iota/sdk';
require('dotenv').config({ path: '.env' });

// Run with command:
// yarn run-example ./client/14_build_foundry_output.ts

// Build a foundry output
async function run() {
    initLogger();
    if (!process.env.NODE_URL) {
        throw new Error('.env NODE_URL is undefined, see .env.example');
    }

    const client = new Client({
        nodes: [process.env.NODE_URL],
    });

    try {
        if (!process.env.NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC_1) {
            throw new Error('.env mnemonic is undefined, see .env.example');
        }

        const aliasId =
            '0xff311f59790ccb85343a36fbac2f06d233734794404142b308c13f2c616935b5';

        const foundryOutput = await client.buildFoundryOutput({
            serialNumber: 0,
            // 10 hex encoded
            tokenScheme: new SimpleTokenScheme('0xa', '0x0', '0xa'),
            amount: '1000000',
            unlockConditions: [
                new ImmutableAliasAddressUnlockCondition(
                    new AliasAddress(aliasId),
                ),
            ],
        });

        console.log(foundryOutput);
        process.exit();
    } catch (error) {
        console.error('Error: ', error);
    }
}

run();
