use pinocchio::{address::Address, entrypoint, error::ProgramError, AccountView, ProgramResult};

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Address,
    accounts: &mut [AccountView],
    _instruction_data: &[u8],
) -> ProgramResult {
    if accounts.is_empty() {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let counter_account = &mut accounts[0];

    if !counter_account.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }

    let mut data = counter_account.try_borrow_mut()?;
    if data.len() < 8 {
        return Err(ProgramError::AccountDataTooSmall);
    }

    let mut count = u64::from_le_bytes(data[0..8].try_into().unwrap());
    count += 1;
    data[0..8].copy_from_slice(&count.to_le_bytes());

    Ok(())
}
