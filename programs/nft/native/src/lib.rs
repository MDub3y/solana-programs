use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::entrypoint;
use solana_program::program_error::ProgramError;
use solana_program::rent::Rent;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

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

    Ok(())
}
