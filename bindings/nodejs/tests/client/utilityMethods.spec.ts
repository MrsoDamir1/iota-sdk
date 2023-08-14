// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

import { describe, it } from '@jest/globals';
import 'reflect-metadata';
import 'dotenv/config';

import { Client, SecretManager, Utils } from '../../';
import '../customMatchers';
import { SlotCommitment } from '../../out/types/block/slot';

const offlineClient = new Client({});

describe('Client utility methods', () => {
    // Requires "stronghold" in cargo toml iota-client features
    it.skip('generates and stores mnemonic', async () => {
        const mnemonic = Utils.generateMnemonic();

        const strongholdSecretManager = {
            stronghold: {
                password: 'some_hopefully_secure_password',
                snapshotPath: './stronghold',
            },
        };
        await expect(
            new SecretManager(strongholdSecretManager).storeMnemonic(mnemonic),
        ).resolves.toBe(null);
    });

    it('invalid mnemonic error', () => {
        try {
            Utils.verifyMnemonic('invalid mnemonic '.repeat(12));
            throw 'should error';
        } catch (e: any) {
            expect(e.payload.error).toContain('NoSuchWord');
        }
    });

    it('converts address to hex and bech32', async () => {
        const address =
            'rms1qpllaj0pyveqfkwxmnngz2c488hfdtmfrj3wfkgxtk4gtyrax0jaxzt70zy';
        const hexAddress = Utils.bech32ToHex(address);

        expect(hexAddress.slice(0, 2)).toBe('0x');

        const bech32Address = await offlineClient.hexToBech32(
            hexAddress,
            'rms',
        );

        expect(bech32Address).toBe(address);
    });

    it('converts hex public key to bech32 address', async () => {
        const hexPublicKey =
            '0x2baaf3bca8ace9f862e60184bd3e79df25ff230f7eaaa4c7f03daa9833ba854a';

        const address = Utils.hexPublicKeyToBech32Address(hexPublicKey, 'rms');

        expect(address).toBeValidAddress();
    });

    it('validates address', async () => {
        const address =
            'rms1qpllaj0pyveqfkwxmnngz2c488hfdtmfrj3wfkgxtk4gtyrax0jaxzt70zy';

        const isAddressValid = Utils.isAddressValid(address);

        expect(isAddressValid).toBeTruthy();
    });

    it('hash output id', async () => {
        const outputId =
            '0x00000000000000000000000000000000000000000000000000000000000000000000';

        const aliasId = Utils.computeAliasId(outputId);

        expect(aliasId).toBe(
            '0xcf077d276686ba64c0404b9eb2d15556782113c5a1985f262b70f9964d3bbd7f',
        );
    });

    it('alias id to address', async () => {
        const aliasId =
            '0xcf077d276686ba64c0404b9eb2d15556782113c5a1985f262b70f9964d3bbd7f';

        const aliasAddress = await offlineClient.aliasIdToBech32(
            aliasId,
            'rms',
        );

        expect(aliasAddress).toBe(
            'rms1pr8swlf8v6rt5exqgp9eavk324t8sggnckseshex9dc0n9jd8w7h7wcnhn7',
        );
    });

    it('compute foundry id', async () => {
        const aliasId =
            '0xcf077d276686ba64c0404b9eb2d15556782113c5a1985f262b70f9964d3bbd7f';
        const serialNumber = 0;
        const tokenSchemeType = 0;

        const foundryId = Utils.computeFoundryId(
            aliasId,
            serialNumber,
            tokenSchemeType,
        );

        expect(foundryId).toBe(
            '0x08cf077d276686ba64c0404b9eb2d15556782113c5a1985f262b70f9964d3bbd7f0000000000',
        );
    });

    it('sign and verify Ed25519', async () => {
        const secretManager = {
            mnemonic: Utils.generateMnemonic(),
        };

        // `IOTA` hex encoded
        const message = '0x494f5441';
        const signature = await new SecretManager(secretManager).signEd25519(
            message,
            {
                coinType: 4218,
                account: 0,
                change: 0,
                addressIndex: 0,
            },
        );

        const validSignature = Utils.verifyEd25519Signature(signature, message);
        expect(validSignature).toBeTruthy();
    });

    it('slot commitment id', async () => {
        let slotCommitment = new SlotCommitment(
            BigInt(10),
            "0x20e07a0ea344707d69a08b90be7ad14eec8326cf2b8b86c8ec23720fab8dcf8ec43a30e4a8cc3f1f",
            "0xcf077d276686ba64c0404b9eb2d15556782113c5a1985f262b70f9964d3bbd7f",
            BigInt(5)
        );
        let id = Utils.computeSlotCommitmentId(slotCommitment);
        expect(id).toBe(
            '0xb485446277cc5111d54f443b46d886945d4af64e53c2f04064a7c2ea88fa4e020a00000000000000'
        );
    });
});
