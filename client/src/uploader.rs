use {
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        instruction::{AccountMeta, Instruction},
        message::Message,
        pubkey::Pubkey,
        signer::keypair::Keypair,
        signer::Signer,
        system_program,
        transaction::Transaction,
    },
};

/*
 * TASKS:
 *  - Split uploaded data into transaction pieces.
 *  - Align derived account computation with on-chain program.
 *  - Implement e2e test:
 *      - Upload multi-piece blob (e.g. 8KB).
 *      - Fetch corresponding Account.
 *      - Verify that data on that account matches originally uploaded blob.
 */

pub async fn upload(
    solana_client: &RpcClient,
    program_id: &Pubkey,
    author: &Keypair,
    data: &[u8],
) -> anyhow::Result<()> {
    // Data Bucket Account
    let (data_bucket_account_pubkey, bump_seed) = Pubkey::find_program_address(
        &[b"solana-data-packer".as_ref(), author.pubkey().as_ref()],
        program_id,
    );

    let data = data.to_vec();
    let (mut chunk, mut data) = data.split_at(768);

    println!("Saving data to account: {:?}", data_bucket_account_pubkey);

    // First create the data bucket.
    let serialized_bucket = bincode::serialize(
        &solana_data_packer_onchain_program::instruction::ProgramInstruction::CreateBucket {
            data: chunk.to_vec(),
            bump_seed,
        },
    )?;

    let instruction = Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(author.pubkey(), true),
            AccountMeta::new(author.pubkey(), true),
            AccountMeta::new(data_bucket_account_pubkey, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: serialized_bucket,
    };

    let latest_blockhash = solana_client
        .get_latest_blockhash()
        .expect("failed to fetch latest blockhash");

    let message = Message::new(&[instruction], Some(&author.pubkey()));
    let transaction = Transaction::new(&[author, author], message, latest_blockhash);

    send_transaction(solana_client, transaction).await?;

    // Then send rest of the data.
    while !data.is_empty() {
        (chunk, data) = data.split_at(768);

        let serialized_bucket = bincode::serialize(&solana_data_packer_onchain_program::instruction::ProgramInstruction::AppendIntoBucket{
            data: chunk.to_vec(),
        })?;

        let instruction = Instruction {
            program_id: *program_id,
            accounts: vec![
                AccountMeta::new(author.pubkey(), true),
                AccountMeta::new(author.pubkey(), true),
                AccountMeta::new(data_bucket_account_pubkey, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: serialized_bucket,
        };

        let message = Message::new(&[instruction], Some(&author.pubkey()));
        let transaction = Transaction::new(&[author, author], message, latest_blockhash);

        send_transaction(solana_client, transaction).await?;
    }

    println!(
        "Verification stored at Account: {:?}",
        data_bucket_account_pubkey
    );

    Ok(())
}

async fn send_transaction(
    solana_client: &RpcClient,
    transaction: Transaction,
) -> solana_client::client_error::Result<()> {
    println!("Sending transaction...");
    let result = solana_client.send_and_confirm_transaction_with_spinner(&transaction);
    println!("Solana onchain program result: {:?}", result);
    result.map(|_| ())
}
