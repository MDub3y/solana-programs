use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;

    if !counter_account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    let mut data = counter_account.try_borrow_mut_data()?;
    if data.len() < 8 {
        return Err(ProgramError::AccountDataTooSmall);
    }

    let mut count = u64::from_le_bytes(data[0..8].try_into().unwrap());
    count += 1;
    data[0..8].copy_from_slice(&count.to_le_bytes());

    Ok(())
}
