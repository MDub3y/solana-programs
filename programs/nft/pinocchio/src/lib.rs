#![no_std]

use pinocchio::{
    cpi::invoke,
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    no_allocator, nostd_panic_handler, program_entrypoint,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};
use solana_program::{pubkey::Pubkey, system_program};

program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

pub fn process_instruction(
    _program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let [authority, nft_mint, token_acccount, token_program, associated_token_program, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if instruction_data.len() < 20 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let mut cur = 0;

    let name_len = u32::from_le_bytes(
        instruction_data[cur..cur + 4]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    ) as usize;
    cur += 4;
    let name = &instruction_data[cur..cur + name_len];
    cur += name_len;

    let symbol_len = u32::from_le_bytes(
        instruction_data[cur..cur + 4]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    ) as usize;
    cur += 4;
    let symbol = &instruction_data[cur..cur + symbol_len];
    cur += symbol_len;

    let uri_len = u32::from_le_bytes(
        instruction_data[cur..cur + 4]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    ) as usize;
    cur += 4;
    let uri = &instruction_data[cur..cur + uri_len];

    // compute account boundary parameters upfront
    // base mint: 165B | AccountType: 1B | Metadata Pointer: 24B | Token Metadata : 12B
    let metadata_payload_len = 4 + name.len() + 4 + symbol.len() + 4 + uri.len();
    let total_account_space = 165 + 1 + 24 + 12 + metadata_payload_len;

    let rent = Rent::get()?;
    let lamports_required = rent.try_minimum_balance(total_account_space)?;

    // 1. CPI -> System Program: Allocate account with full space
    let mut create_data = [0u8; 52];
    create_data[0..4].copy_from_slice(&0u32.to_le_bytes());
    create_data[4..12].copy_from_slice(&lamports_required.to_le_bytes());
    create_data[12..20].copy_from_slice(&(total_account_space as u64).to_le_bytes());
    create_data[20..52].copy_from_slice(token_program.address().as_ref());

    invoke(
        &InstructionView {
            program_id: system_program.address(),
            data: &create_data,
            accounts: &[
                InstructionAccount::writable_signer(authority.address()),
                InstructionAccount::writable_signer(nft_mint.address()),
            ],
        },
        &[authority.clone(), nft_mint.clone()],
    )?;

    // 2. CPI -> Token-2022: Initialize Metadata Pointer Extension
    let mut meta_ptr_data = [0u8; 69];
    meta_ptr_data[0] = 39;
    meta_ptr_data[1] = 0;
    meta_ptr_data[2..6].copy_from_slice(&1u8.to_le_bytes()); // Some(Authority)
    meta_ptr_data[6..38].copy_from_slice(authority.address().as_ref());
    meta_ptr_data[38..42].copy_from_slice(&1u32.to_le_bytes()); // Some(MetadataAddress)
    meta_ptr_data[42..74].copy_from_slice(nft_mint.address().as_ref());

    invoke(
        &InstructionView {
            program_id: token_program.address(),
            data: &meta_ptr_data,
            accounts: &[InstructionAccount::writable(nft_mint.address())],
        },
        &[nft_mint.clone()],
    )?;

    Ok(())
}
