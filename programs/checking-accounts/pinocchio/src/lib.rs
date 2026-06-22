#![no_std]

use pinocchio::{
    entrypoint, error::ProgramError, nostd_panic_handler, AccountView, Address, ProgramResult,
};
use pinocchio_log::log;

entrypoint!(process_instruction);
nostd_panic_handler!();

fn process_instruction(
    program_id: &Address,
    accounts: &[AccountView],
    _instruction_data: &[u8],
) -> ProgramResult {
    let [payer, account_to_create, account_to_change, system_program] = accounts else {
        log!("This instruction requires 4 accounts:");
        log!("  payer, account_to_create, account_to_change, system_program");
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !payer.is_signer() {
        log!("The program expected the account to be a signer.");
        return Err(ProgramError::MissingRequiredSignature);
    }

    log!("New account: {}", account_to_create.address().as_array());
    if account_to_create.lamports() != 0 {
        log!("The program expected the account to create to not yet be initialized.");
        return Err(ProgramError::AccountAlreadyInitialized);
    };
    // (Create account...)

    log!(
        "Account to change: {}",
        account_to_change.address().as_array()
    );
    if account_to_change.lamports() == 0 {
        log!("The program expected the account to change to be initialized.");
        return Err(ProgramError::UninitializedAccount);
    };

    if !account_to_change.owned_by(program_id) {
        log!("Account to change does not have the correct program id.");
        return Err(ProgramError::IncorrectProgramId);
    };

    if system_program.address() != &pinocchio_system::ID {
        return Err(ProgramError::IncorrectProgramId);
    };

    Ok(())
}
