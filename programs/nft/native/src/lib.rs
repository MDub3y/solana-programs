use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::entrypoint;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token_2022::{
    extension::{metadata_pointer::instruction::initialize as init_meta_ptr, ExtensionType},
    instruction::{initialize_mint2, mint_to},
    state::Mint,
};
use spl_token_metadata_interface::instruction::initialize as init_metadata;

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct MintNftArgs {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let authority = next_account_info(account_info_iter)?;
    let nft_mint = next_account_info(account_info_iter)?;
    let token_account = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    let associated_token_program = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    if instruction_data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData);
    }
    let args = MintNftArgs::try_from_slice(&instruction_data[8..])?;

    let extension_types = [ExtensionType::MetadataPointer, ExtensionType::TokenMetadata];
    let base_extension_len = ExtensionType::try_calculate_account_len::<Mint>(&extension_types)?;

    let string_payload_len = 4 + args.name.len() + 4 + args.symbol.len() + 4 + args.uri.len();
    let total_account_space = base_extension_len + string_payload_len;

    let lamports_required = Rent::get()?.minimum_balance(total_account_space);

    // 1: CPI -> System Program: Create Mint Account with exact final size
    invoke(
        &system_instruction::create_account(
            authority.key,
            nft_mint.key,
            lamports_required,
            total_account_space as u64,
            token_program.key,
        ),
        &[authority.clone(), nft_mint.clone(), system_program.clone()],
    )?;

    // 2: CPI -> Token-2022: Initialize Metadata Pointer Signpost Header
    invoke(
        &init_meta_ptr(
            token_program.key,
            nft_mint.key,
            Some(*authority.key),
            Some(*nft_mint.key),
        )?,
        &[nft_mint.clone()],
    )?;

    // 3: CPI -> Token-2022: Initialize 165-Byte Base Mint Properties
    invoke(
        &initialize_mint2(token_program.key, nft_mint.key, authority.key, None, 0)?,
        &[nft_mint.clone()],
    )?;

    // 4: CPI -> Associated Token Program: Create Recipient Token Vault
    invoke(
        &create_associated_token_account(
            authority.key,
            authority.key,
            nft_mint.key,
            token_program.key,
        ),
        &[
            authority.clone(),
            token_account.clone(),
            authority.clone(),
            nft_mint.clone(),
            system_program.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ],
    )?;

    // 5: CPI -> Token-2022: Write Dynamic Metadata Strings into TLV Space
    invoke(
        &init_metadata(
            token_program.key,
            nft_mint.key,
            authority.key,
            nft_mint.key,
            authority.key,
            args.name,
            args.symbol,
            args.uri,
        ),
        &[nft_mint.clone(), authority.clone()],
    )?;

    // 6: CPI -> Token-2022: Mint exactly 1 Token to user wallet
    invoke(
        &mint_to(
            token_program.key,
            nft_mint.key,
            token_account.key,
            authority.key,
            &[],
            1,
        )?,
        &[nft_mint.clone(), token_account.clone(), authority.clone()],
    )?;

    Ok(())
}
