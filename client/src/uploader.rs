use {
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        account_info::AccountInfo,
        instruction::{AccountMeta, Instruction},
        message::Message,
        pubkey::Pubkey,
        signer::Signer,
        signer::keypair::Keypair,
        transaction::Transaction,
    },
};

/*
 * TASKS:
 *  - Build CLI uploader for testing.
 *  - Split uploaded data into transaction pieces.
 *  - Align derived account computation with on-chain program.
 *  - Implement e2e test:
 *      - Upload multi-piece blob (e.g. 8KB).
 *      - Fetch corresponding Account.
 *      - Verify that data on that account matches originally uploaded blob.
 */

pub async fn upload(solana_client: &RpcClient, program_id: &Pubkey, author: Keypair, data: &[u8], _dst: AccountInfo<'_>) -> anyhow::Result<()> {
        // Data Bucket Account
        let (state_account_pubkey, _) = Pubkey::find_program_address(
            &[
                b"solana-data-packer".as_ref(),
                author.pubkey().as_ref(),
            ],
            program_id,
        );

        let serialized_bucket = bincode::serialize(&solana_data_packer_onchain_program::instruction::ProgramInstruction::CreateBucket{
                data: data.to_vec(),
                bump_seed: 42,
            })?;

        let instruction = Instruction {
            program_id: *program_id,
            accounts: vec![
                AccountMeta::new(state_account_pubkey, false),
                AccountMeta::new(author.pubkey(), true),
            ],
            data: serialized_bucket,
        };

        let latest_blockhash = solana_client
            .get_latest_blockhash()
            .expect("failed to fetch latest blockhash");

        let message = Message::new(&[instruction], Some(&author.pubkey()));
        let transaction = Transaction::new(&[&author], message, latest_blockhash);

        send_transaction(solana_client, transaction).await?;
        println!("Verification stored at Account: {:?}", state_account_pubkey);
        Ok(())
    }
 


async fn send_transaction(
        solana_client: &RpcClient,
        transaction: Transaction,
    ) -> solana_client::client_error::Result<()> {
        println!("Sending transaction...");
        let result = solana_client.send_transaction(&transaction);
        println!("Solana onchain program result: {:?}", result);
        Ok(())
    }
