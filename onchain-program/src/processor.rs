use {
    crate::{instruction::ProgramInstruction, state},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        clock::Clock,
        entrypoint,
        entrypoint::ProgramResult,
        program::invoke_signed,
        program_error::ProgramError,
        program_utils::limited_deserialize, pubkey::Pubkey,
        rent::Rent,
        msg,
        system_instruction,
        sysvar::Sysvar,
    },
};

// Following copied from solana/sdk/src/packet.rs:
//
/// Maximum over-the-wire size of a Transaction
///   1280 is IPv6 minimum MTU
///   40 bytes is the size of the IPv6 header
///   8 bytes is the size of the fragment header
pub const SOLANA_PACKET_DATA_SIZE: u64 = 1280 - 40 - 8;

// Declare and export the program's entrypoint.
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match limited_deserialize(instruction_data, SOLANA_PACKET_DATA_SIZE).map_err(|_| { ProgramError::InvalidInstructionData })? {
        ProgramInstruction::CreateBucket {
            data,
            bump_seed,
        } => Processor::create_bucket(
            program_id,
            accounts,
            data,
            bump_seed,
        ),
        ProgramInstruction::AppendIntoBucket { data } =>
            Processor::append_into_bucket(
                program_id,
                accounts,
                data,
            ),
    }
}

pub struct Processor;
impl Processor {
    fn create_bucket(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        data: Vec<u8>,
        bump_seed: u8,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let authority_account = next_account_info(account_info_iter)?;
        let payer_account = next_account_info(account_info_iter)?;
        let data_bucket_account = next_account_info(account_info_iter)?;

        let authority_key = *authority_account.signer_key().ok_or_else(|| {
            msg!("Authority account must be a signer");
            ProgramError::MissingRequiredSignature
        })?;

        let payer_key = *payer_account.signer_key().ok_or_else(|| {
            msg!("Payer account must be a signer");
            ProgramError::MissingRequiredSignature
        })?;

        // Use a derived address to ensure that an address table can never be
        // initialized more than once at the same address.
        let derived_data_bucket_key = Pubkey::create_program_address(
            &[
                authority_key.as_ref(),
                &[bump_seed],
            ],
            program_id,
        )?;

        let data_bucket_key = *data_bucket_account.unsigned_key();
        if data_bucket_key != derived_data_bucket_key {
            msg!(
                "Table address must match derived address: {}",
                derived_data_bucket_key
            );
            return Err(ProgramError::InvalidArgument);
        }

        let current_slot = Clock::get()?.slot;
        let data_bucket_len = 72 + data.len();
        let data_bucket = state::DataBucket {
            meta: state::DataBucketMeta {
                last_updated_slot: current_slot,
                authority: Some(authority_key),
            },
            data,
        };

        let rent = Rent::default();
        let required_lamports = rent
            .minimum_balance(data_bucket_len)
            .max(1)
            .saturating_sub(data_bucket_account.lamports());

        if required_lamports > 0 {
            invoke_signed(
                &system_instruction::transfer(&payer_key, &data_bucket_key, required_lamports),
                &[
                    payer_account.clone(),
                    data_bucket_account.clone(),
                ],
               &[&[b"solana-data-packer", payer_key.as_ref()]],
            )?;
        }

        invoke_signed(
            &system_instruction::allocate(&data_bucket_key, data_bucket_len as u64),
            &[data_bucket_account.clone()],
            &[&[b"solana-data-packer", data_bucket_key.as_ref()]],
        )?;

        invoke_signed(
            &system_instruction::assign(&data_bucket_key, program_id),
            &[data_bucket_account.clone()],
            &[&[b"solana-data-packer", data_bucket_key.as_ref()]],
        )?;

        // Finally store the data in the bucket.
        data_bucket_account.serialize_data(&data_bucket).map_err(|_| { ProgramError::InvalidAccountData })
    }

    fn append_into_bucket(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        _data: Vec<u8>,
    ) -> ProgramResult {
        Ok(())
    }
}
