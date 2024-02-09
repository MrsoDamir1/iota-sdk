// Copyright 2023 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    client::{api::PreparedTransactionData, secret::SecretManage},
    types::block::{
        address::Address,
        context_input::{BlockIssuanceCreditContextInput, CommitmentContextInput},
        output::{
            feature::{
                BlockIssuerFeature, BlockIssuerKey, BlockIssuerKeySource, BlockIssuerKeys,
                Ed25519PublicKeyHashBlockIssuerKey,
            },
            unlock_condition::AddressUnlockCondition,
            AccountId, AccountOutput, OutputId,
        },
    },
    wallet::{
        core::SecretData,
        operations::transaction::{TransactionOptions, TransactionWithMetadata},
        Error, Result, Wallet,
    },
};

impl<S: SecretManage + 'static> Wallet<SecretData<S>> {
    /// Transitions an implicit account to an account.
    pub async fn implicit_account_transition(
        &self,
        output_id: &OutputId,
        key_source: impl Into<BlockIssuerKeySource<S::GenerationOptions>> + Send,
    ) -> Result<TransactionWithMetadata> {
        let issuer_id = AccountId::from(output_id);

        self.sign_and_submit_transaction(
            self.prepare_implicit_account_transition(output_id, key_source).await?,
            issuer_id,
            None,
        )
        .await
    }

    /// Prepares to transition an implicit account to an account.
    pub async fn prepare_implicit_account_transition(
        &self,
        output_id: &OutputId,
        key_source: impl Into<BlockIssuerKeySource<S::GenerationOptions>> + Send,
    ) -> Result<PreparedTransactionData> {
        let wallet_data = self.data().await;
        let implicit_account_data = wallet_data
            .unspent_outputs
            .get(output_id)
            .ok_or(Error::ImplicitAccountNotFound)?;
        let implicit_account = if implicit_account_data.output.is_implicit_account() {
            implicit_account_data.output.as_basic()
        } else {
            return Err(Error::ImplicitAccountNotFound);
        };
        let ed25519_address = *implicit_account
            .address()
            .as_implicit_account_creation()
            .ed25519_address();

        let block_issuer_key = BlockIssuerKey::from(match key_source.into() {
            BlockIssuerKeySource::ImplicitAccountAddress => Ed25519PublicKeyHashBlockIssuerKey::new(*ed25519_address),
            BlockIssuerKeySource::PublicKey(public_key) => {
                Ed25519PublicKeyHashBlockIssuerKey::from_public_key(public_key)
            }
            BlockIssuerKeySource::Options(options) => Ed25519PublicKeyHashBlockIssuerKey::from_public_key(
                self.secret_manager().read().await.generate(&options).await?,
            ),
        });

        let account_id = AccountId::from(output_id);
        let account = AccountOutput::build_with_amount(implicit_account.amount(), account_id)
            .with_mana(implicit_account_data.output.available_mana(
                &self.client().get_protocol_parameters().await?,
                implicit_account_data.output_id.transaction_id().slot_index(),
                self.client().get_slot_index().await?,
            )?)
            .with_unlock_conditions([AddressUnlockCondition::from(Address::from(ed25519_address))])
            .with_features([BlockIssuerFeature::new(
                u32::MAX,
                BlockIssuerKeys::from_vec(vec![block_issuer_key])?,
            )?])
            .finish_output()?;

        drop(wallet_data);

        // TODO https://github.com/iotaledger/iota-sdk/issues/1740
        let issuance = self.client().get_issuance().await?;

        let transaction_options = TransactionOptions {
            context_inputs: Some(vec![
                // TODO Remove in https://github.com/iotaledger/iota-sdk/pull/1872
                CommitmentContextInput::new(issuance.latest_commitment.id()).into(),
                BlockIssuanceCreditContextInput::new(account_id).into(),
            ]),
            custom_inputs: Some(vec![*output_id]),
            ..Default::default()
        };

        self.prepare_transaction(vec![account], transaction_options.clone())
            .await
    }
}
