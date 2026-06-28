use anchor_lang::prelude::*;
use anchor_spl::token_2022::Token2022;

declare_id!("GovAnchor1111111111111111111111111111111111");

#[program]
pub mod governance_anchor {
    use super::*;

    pub fn cast_vote(ctx: Context<CastVote>, proposal_id: u64, weight: u64) -> Result<()> {
        msg!(
            "Vote Confirmed for proposal {}: weight {}",
            proposal_id,
            weight
        );
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(proposal_id: u64)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    #[account(
        mut,
        owner = id() @ ErrorCode::ConstraintOwner
    )]
    pub proposal_state: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ReclaimSurplusSol<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"treasury_vault"],
        bump,
    )]
    pub treasury_pda: UncheckedAccount<'info>,

    #[account(mut)]
    pub stranded_source: UncheckedAccount<'info>,

    #[account(mut)]
    pub destination_wallet: UncheckedAccount<'info>,

    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}
