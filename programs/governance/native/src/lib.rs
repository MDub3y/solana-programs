use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::next_account_info;
use solana_program::entrypoint;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

#[derive(BorshDeserialize, BorshSerialize)]
pub enum GovernanceInstruction {
    CastVote { proposal_id: u64, weight: u64 },
    ReclaimSurplusSol { bump: u8 },
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let mut data_ptr = instruction_data;
    let instruction = GovernanceInstruction::deserialize(&mut data_ptr)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        GovernanceInstruction::CastVote {
            proposal_id,
            weight,
        } => process_cast_vote(program_id, accounts, proposal_id, weight),
        GovernanceInstruction::ReclaimSurplusSol { bump } => {
            process_reclaim_surplus_sol(program_id, accounts, bump)
        }
    }
}

fn process_cast_vote(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    proposal_id: u64,
    weight: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let voter = next_account_info(account_iter)?;
    let proposal_state = next_account_info(account_iter)?;

    // GUARD: Immediate ownership verification
    // This protects against deferred validation bypass vulnerabilities
    if proposal_state.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !voter.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    solana_program::msg!(
        "Vote verified for proposal {}: weight {}",
        proposal_id,
        weight
    );
    Ok(())
}

fn process_reclaim_surplus_sol(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    bump: u8,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let treasury_pda = next_account_info(account_iter)?;
    let stranded_source = next_account_info(account_iter)?;
    let destination_wallet = next_account_info(account_iter)?;
    let token_2022_program = next_account_info(account_iter)?;

    // Derive and verify the treasury PDA to ensure it matches the expected seeds
    let expected_treasury_pda =
        Pubkey::create_program_address(&[b"treasury_vault", &[bump]], program_id)?;

    if treasury_pda.key != &expected_treasury_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    solana_program::msg!("Assembling low-level WithdrawExcessLamports parameters...");

    // Discriminator for WithdrawExcessLamports is 38
    let mut instruction_data = [0u8; 1];
    instruction_data[0] = 38;

    let cpi_instruction = Instruction {
        program_id: *token_2022_program.key,
        accounts: vec![
            AccountMeta::new(*stranded_source.key, false),
            AccountMeta::new(*destination_wallet.key, false),
            AccountMeta::new_readonly(*treasury_pda.key, true),
        ],
        data: instruction_data.to_vec(),
    };

    // Execute the CPI using the PDA seeds to authorize the withdrawal
    invoke_signed(
        &cpi_instruction,
        &[
            stranded_source.clone(),
            destination_wallet.clone(),
            treasury_pda.clone(),
            token_2022_program.clone(),
        ],
        &[&[b"treasury_vault", &[bump]]],
    )?;

    solana_program::msg!("Surplus lamports successfully reclaimed to destination wallet.");
    Ok(())
}
