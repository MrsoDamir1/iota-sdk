// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::str::FromStr;

use crypto::keys::bip44::Bip44;
use iota_sdk::{
    client::{
        api::{transaction::validate_signed_transaction_payload_length, verify_semantic, PreparedTransactionData},
        constants::SHIMMER_COIN_TYPE,
        secret::{PublicKeyOptions, SecretManageExt, SignTransaction},
        Result,
    },
    types::block::{
        address::{Address, Ed25519Address, NftAddress},
        input::{Input, UtxoInput},
        output::NftId,
        payload::{signed_transaction::Transaction, SignedTransactionPayload},
        protocol::protocol_parameters,
        slot::SlotIndex,
        unlock::{SignatureUnlock, Unlock},
    },
};
use pretty_assertions::assert_eq;

use crate::client::{
    build_inputs, build_outputs, random_mnemonic_secret_manager,
    Build::{Basic, Nft},
    NFT_ID_1,
};

#[tokio::test]
async fn nft_reference_unlocks() -> Result<()> {
    let secret_manager = random_mnemonic_secret_manager()?;

    let address_0 = Address::from(
        secret_manager
            .generate::<Ed25519Address>(&PublicKeyOptions::new(SHIMMER_COIN_TYPE))
            .await?,
    );

    let protocol_parameters = protocol_parameters();
    let nft_id = NftId::from_str(NFT_ID_1)?;
    let nft_address = Address::Nft(NftAddress::new(nft_id));
    let slot_index = SlotIndex::from(10);

    let inputs = build_inputs(
        [
            Nft {
                amount: 1_000_000,
                nft_id: nft_id,
                address: address_0.clone(),
                sender: None,
                issuer: None,
                sdruc: None,
                expiration: None,
            },
            Basic {
                amount: 1_000_000,
                address: nft_address.clone(),
                native_token: None,
                sender: None,
                sdruc: None,
                timelock: None,
                expiration: None,
            },
            Basic {
                amount: 1_000_000,
                address: nft_address.clone(),
                native_token: None,
                sender: None,
                sdruc: None,
                timelock: None,
                expiration: None,
            },
        ],
        Some(slot_index),
    );

    let outputs = build_outputs([
        Nft {
            amount: 1_000_000,
            nft_id: nft_id,
            address: address_0,
            sender: None,
            issuer: None,
            sdruc: None,
            expiration: None,
        },
        Basic {
            amount: 2_000_000,
            address: nft_address,
            native_token: None,
            sender: None,
            sdruc: None,
            timelock: None,
            expiration: None,
        },
    ]);

    let transaction = Transaction::builder(protocol_parameters.network_id())
        .with_inputs(
            inputs
                .iter()
                .map(|i| Input::Utxo(UtxoInput::from(*i.output_metadata.output_id())))
                .collect::<Vec<_>>(),
        )
        .with_outputs(outputs)
        .with_creation_slot(slot_index + 1)
        .finish_with_params(&protocol_parameters)?;

    let prepared_transaction_data = PreparedTransactionData {
        transaction,
        inputs_data: inputs,
        remainders: Vec::new(),
        mana_rewards: Default::default(),
    };

    let signing_options = Bip44::new(SHIMMER_COIN_TYPE);

    let unlocks = secret_manager
        .transaction_unlocks(&prepared_transaction_data, &protocol_parameters, &signing_options)
        .await?;

    assert_eq!(unlocks.len(), 3);
    assert_eq!((*unlocks).first().unwrap().kind(), SignatureUnlock::KIND);
    match (*unlocks).get(1).unwrap() {
        Unlock::Nft(a) => {
            assert_eq!(a.index(), 0);
        }
        _ => panic!("Invalid unlock"),
    }
    match (*unlocks).get(2).unwrap() {
        Unlock::Nft(a) => {
            assert_eq!(a.index(), 0);
        }
        _ => panic!("Invalid unlock"),
    }

    let tx_payload = SignedTransactionPayload::new(prepared_transaction_data.transaction.clone(), unlocks)?;

    validate_signed_transaction_payload_length(&tx_payload)?;

    let conflict = verify_semantic(
        &prepared_transaction_data.inputs_data,
        &tx_payload,
        prepared_transaction_data.mana_rewards,
        protocol_parameters,
    )?;

    if let Some(conflict) = conflict {
        panic!("{conflict:?}, with {tx_payload:#?}");
    }

    Ok(())
}
