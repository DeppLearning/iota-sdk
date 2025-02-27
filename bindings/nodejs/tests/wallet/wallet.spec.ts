// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

const { Wallet, CoinType } = require('../../lib/');

describe('Wallet', () => {
    it('create account', async () => {
        let storagePath = "test-create-account";
        removeDir(storagePath)

        const walletOptions = {
            storagePath: './test-create-account',
            clientOptions: {
                nodes: ["https://api.testnet.shimmer.network"],
            },
            coinType: CoinType.Shimmer,
            secretManager: {
                Stronghold: {
                    snapshotPath: `./test-create-account/wallet.stronghold`,
                    password: `A12345678*`,
                },
            },
        };

        const wallet = new Wallet(walletOptions);
        await wallet.storeMnemonic("vital give early extra blind skin eight discover scissors there globe deal goat fat load robot return rate fragile recycle select live ordinary claim");

        const account = await wallet.createAccount({
            alias: 'Alice',
        });

        expect(account.getMetadata().index).toStrictEqual(0);

        await wallet.destroy()
        removeDir(storagePath)
    });

    it('generate address', async () => {
        let storagePath = "test-generate-address";
        removeDir(storagePath)

        const walletOptions = {
            storagePath,
            clientOptions: {
                nodes: ["https://api.testnet.shimmer.network"],
            },
            coinType: CoinType.Shimmer,
            secretManager: {
                Stronghold: {
                    snapshotPath: `./test-generate-address/wallet.stronghold`,
                    password: `A12345678*`,
                },
            },
        };

        const wallet = new Wallet(walletOptions);
        await wallet.storeMnemonic("vital give early extra blind skin eight discover scissors there globe deal goat fat load robot return rate fragile recycle select live ordinary claim");

        const address = await wallet.generateAddress(
            0,
            0,
            { internal: false, ledgerNanoPrompt: false },
            "rms"
        );

        expect(address).toStrictEqual("rms1qpqzgvcehafmlxh87zrf9w8ck8q2kw5070ztf68ylhzk89en9a4fy5jqrg8");

        const anotherAddress = await wallet.generateAddress(
            10,
            10,
            { internal: true, ledgerNanoPrompt: false },
            "tst"
        );

        expect(anotherAddress).toStrictEqual("tst1qzp37j45rkfmqn05fapq66vyw0vkmz5zqhmeuey5fked0wt4ry43jeqp2wv");

        await wallet.destroy()
        removeDir(storagePath)
    });

    it('recreate wallet', async () => {
        let storagePath = "test-recreate-wallet";
        removeDir(storagePath)

        const walletOptions = {
            storagePath,
            clientOptions: {
                nodes: ["https://api.testnet.shimmer.network"],
            },
            coinType: CoinType.Shimmer,
            secretManager: {
                Stronghold: {
                    snapshotPath: `./test-create-account/wallet.stronghold`,
                    password: `A12345678*`,
                },
            },
        };

        const wallet = new Wallet(walletOptions);
        await wallet.storeMnemonic("vital give early extra blind skin eight discover scissors there globe deal goat fat load robot return rate fragile recycle select live ordinary claim");

        const account = await wallet.createAccount({
            alias: 'Alice',
        });

        expect(account.getMetadata().index).toStrictEqual(0);

        const client = await wallet.getClient();

        const localPoW = await client.getLocalPow();
        expect(localPoW).toBeTruthy();

        await wallet.destroy();

        const recreatedWallet = new Wallet(walletOptions);
        const accounts = await recreatedWallet.getAccounts();
        expect(accounts.length).toStrictEqual(1);

        await recreatedWallet.destroy()
        removeDir(storagePath)
    });

    it('error after destroy', async () => {
        let storagePath = "test-error-after-destroy";
        removeDir(storagePath)

        const walletOptions = {
            storagePath,
            clientOptions: {
                nodes: ["https://api.testnet.shimmer.network"],
            },
            coinType: CoinType.Shimmer,
            secretManager: {
                Stronghold: {
                    snapshotPath: `./test-error-after-destroy/wallet.stronghold`,
                    password: `A12345678*`,
                },
            },
        };

        const wallet = new Wallet(walletOptions);
        await wallet.storeMnemonic("vital give early extra blind skin eight discover scissors there globe deal goat fat load robot return rate fragile recycle select live ordinary claim");

        const account = await wallet.createAccount({
            alias: 'Alice',
        });

        expect(account.getMetadata().index).toStrictEqual(0);

        await wallet.destroy();
        
        try{
            const accounts = await wallet.getAccounts();
            throw("Should return an error because the wallet got destroyed");
        }catch(err: any){
            expect(err).toContain("Wallet got destroyed");
        }

        try{
            const client = await wallet.getClient();
            throw("Should return an error because the wallet got destroyed");
        }catch(err: any){
            expect(err).toContain("Wallet got destroyed");
        }
        removeDir(storagePath)
    });
})

function removeDir(storagePath: string) {
    const fs = require('fs');
    fs.rmSync(storagePath, { recursive: true, force: true });
}
