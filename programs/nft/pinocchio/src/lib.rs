#![no_std]

use pinocchio::{
    cpi::invoke,
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    no_allocator, nostd_panic_handler, program_entrypoint,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};

program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

pub fn process_instruction(
    _program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let [authority, nft_mint, token_account, token_program, associated_token_program, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let mut cur = 8;

    let name_len_bytes = instruction_data
        .get(cur..cur + 4)
        .ok_or(ProgramError::InvalidInstructionData)?;
    let name_len = u32::from_le_bytes(
        name_len_bytes
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    ) as usize;
    cur += 4;

    let name = instruction_data
        .get(cur..cur + name_len)
        .ok_or(ProgramError::InvalidInstructionData)?;
    cur += name_len;

    let symbol_len_bytes = instruction_data
        .get(cur..cur + 4)
        .ok_or(ProgramError::InvalidInstructionData)?;
    let symbol_len = u32::from_le_bytes(
        symbol_len_bytes
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    ) as usize;
    cur += 4;

    let symbol = instruction_data
        .get(cur..cur + symbol_len)
        .ok_or(ProgramError::InvalidInstructionData)?;
    cur += symbol_len;

    let uri_len_bytes = instruction_data
        .get(cur..cur + 4)
        .ok_or(ProgramError::InvalidInstructionData)?;
    let uri_len = u32::from_le_bytes(
        uri_len_bytes
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    ) as usize;
    cur += 4;

    let uri = instruction_data
        .get(cur..cur + uri_len)
        .ok_or(ProgramError::InvalidInstructionData)?;

    let metadata_payload_len = 4 + name.len() + 4 + symbol.len() + 4 + uri.len();
    let total_account_space = 165 + 1 + 24 + 12 + metadata_payload_len;

    let rent = Rent::get()?;
    let lamports_required = rent.try_minimum_balance(total_account_space)?;

    // STEP 1: CPI -> System Program: Allocate Account with full final size
    let mut create_data = [0u8; 52];
    create_data[0..4].copy_from_slice(&0u32.to_le_bytes());
    create_data[4..12].copy_from_slice(&lamports_required.to_le_bytes());
    create_data[12..20].copy_from_slice(&(total_account_space as u64).to_le_bytes());
    create_data[20..52].copy_from_slice(token_program.address().as_ref());

    invoke(
        &InstructionView {
            program_id: system_program.address(),
            accounts: &[
                InstructionAccount::writable_signer(authority.address()),
                InstructionAccount::writable_signer(nft_mint.address()),
            ],
            data: &create_data,
        },
        &[authority.clone(), nft_mint.clone()],
    )?;

    // STEP 2: CPI -> Token-2022: Initialize Metadata Pointer Extension
    let mut meta_ptr_data = [0u8; 66];
    meta_ptr_data[0] = 39;
    meta_ptr_data[1] = 0;
    meta_ptr_data[2..34].copy_from_slice(authority.address().as_ref());
    meta_ptr_data[34..66].copy_from_slice(nft_mint.address().as_ref());

    invoke(
        &InstructionView {
            program_id: token_program.address(),
            accounts: &[InstructionAccount::writable(nft_mint.address())],
            data: &meta_ptr_data,
        },
        &[nft_mint.clone()],
    )?;

    // STEP 3: CPI -> Token-2022: Initialize Base Mint Layout
    let mut mint_data = [0u8; 38];
    mint_data[0] = 20;
    mint_data[1] = 0;
    mint_data[2..34].copy_from_slice(authority.address().as_ref());

    invoke(
        &InstructionView {
            program_id: token_program.address(),
            accounts: &[InstructionAccount::writable(nft_mint.address())],
            data: &mint_data,
        },
        &[nft_mint.clone()],
    )?;

    // STEP 4: CPI -> Associated Token Account Program: Create User ATA
    invoke(
        &InstructionView {
            program_id: associated_token_program.address(),
            accounts: &[
                InstructionAccount::writable_signer(authority.address()),
                InstructionAccount::writable(token_account.address()),
                InstructionAccount::readonly(authority.address()),
                InstructionAccount::readonly(nft_mint.address()),
                InstructionAccount::readonly(system_program.address()),
                InstructionAccount::readonly(token_program.address()),
            ],
            data: &[],
        },
        &[
            authority.clone(),
            token_account.clone(),
            authority.clone(),
            nft_mint.clone(),
            system_program.clone(),
            token_program.clone(),
        ],
    )?;

    // STEP 5: CPI -> Token-2022: Initialize Metadata Strings
    let mut meta_init_data = [0u8; 1024];
    meta_init_data[0..8].copy_from_slice(&[219, 131, 102, 114, 184, 196, 215, 187]);
    meta_init_data[8..40].copy_from_slice(authority.address().as_ref());
    meta_init_data[40..72].copy_from_slice(nft_mint.address().as_ref());
    meta_init_data[72..104].copy_from_slice(authority.address().as_ref());

    let mut meta_idx = 104;

    meta_init_data[meta_idx..meta_idx + 4].copy_from_slice(&(name.len() as u32).to_le_bytes());
    meta_idx += 4;
    meta_init_data[meta_idx..meta_idx + name.len()].copy_from_slice(name);
    meta_idx += name.len();

    meta_init_data[meta_idx..meta_idx + 4].copy_from_slice(&(symbol.len() as u32).to_le_bytes());
    meta_idx += 4;
    meta_init_data[meta_idx..meta_idx + symbol.len()].copy_from_slice(symbol);
    meta_idx += symbol.len();

    meta_init_data[meta_idx..meta_idx + 4].copy_from_slice(&(uri.len() as u32).to_le_bytes());
    meta_idx += 4;
    meta_init_data[meta_idx..meta_idx + uri.len()].copy_from_slice(uri);
    meta_idx += uri.len();

    invoke(
        &InstructionView {
            program_id: token_program.address(),
            accounts: &[
                InstructionAccount::writable(nft_mint.address()),
                InstructionAccount::readonly_signer(authority.address()),
                InstructionAccount::readonly(nft_mint.address()),
                InstructionAccount::readonly_signer(authority.address()),
            ],
            data: &meta_init_data[..meta_idx],
        },
        &[nft_mint.clone(), authority.clone()],
    )?;

    // STEP 6: CPI -> Token-2022: Mint exactly 1 Token to user vault
    let mut mint_to_data = [0u8; 9];
    mint_to_data[0] = 7;
    mint_to_data[1..9].copy_from_slice(&1u64.to_le_bytes());

    invoke(
        &InstructionView {
            program_id: token_program.address(),
            accounts: &[
                InstructionAccount::writable(nft_mint.address()),
                InstructionAccount::writable(token_account.address()),
                InstructionAccount::readonly_signer(authority.address()),
            ],
            data: &mint_to_data,
        },
        &[nft_mint.clone(), token_account.clone(), authority.clone()],
    )?;

    Ok(())
}
