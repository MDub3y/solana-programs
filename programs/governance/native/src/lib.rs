use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::entrypoint;
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

    Ok(())
}
