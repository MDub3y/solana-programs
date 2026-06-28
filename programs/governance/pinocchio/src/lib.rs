#![no_std]

use pinocchio::{
    cpi::{invoke_signed, Seed, Signer},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    no_allocator, nostd_panic_handler, program_entrypoint, AccountView, Address, ProgramResult,
};

program_entrypoint!(process_instruction);
no_allocator!();
nostd_panic_handler!();

pub fn process_instruction(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let variant = instruction_data
        .get(0)
        .ok_or(ProgramError::InvalidInstructionData)?;

    match variant {
        0 => process_cast_vote(program_id, accounts, instruction_data),
        1 => process_reclaim_surplus_sol(program_id, accounts, instruction_data),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

fn process_cast_vote(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let [voter, proposal_state] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if proposal_state.owner() != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !voter.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let proposal_id_bytes = instruction_data
        .get(1..9)
        .ok_or(ProgramError::InvalidInstructionData)?;
    let weight_bytes = instruction_data
        .get(9..17)
        .ok_or(ProgramError::InvalidInstructionData)?;

    let _proposal_id = u64::from_le_bytes(
        proposal_id_bytes
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );
    let _weight = u64::from_le_bytes(
        weight_bytes
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );

    Ok(())
}

fn process_reclaim_surplus_sol(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let [treasury_pda, stranded_source, destination_wallet, token_2022_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let bump = *instruction_data
        .get(1)
        .ok_or(ProgramError::InvalidInstructionData)?;

    let mut withdraw_payload = [0u8; 1];
    withdraw_payload[0] = 38;

    let bump_binding = [bump];
    let seeds = [Seed::from(b"treasury_vault"), Seed::from(&bump_binding)];
    let signers = Signer::from(&seeds);

    invoke_signed(
        &InstructionView {
            program_id: token_2022_program.address(),
            accounts: &[
                InstructionAccount::writable(stranded_source.address()),
                InstructionAccount::writable(destination_wallet.address()),
            ],
            data: &withdraw_payload,
        },
        &[
            stranded_source.clone(),
            destination_wallet.clone(),
            treasury_pda.clone(),
        ],
        &[signers],
    )?;

    Ok(())
}
