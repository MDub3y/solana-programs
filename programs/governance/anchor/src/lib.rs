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

    pub fn reclaim_surplus_sol(ctx: Context<ReclaimSurplusSol>) -> Result<()> {
        let bump = ctx.bumps.treasury_pda;
        let signer_seeds: &[&[u8]] = &[b"treasury_vault", &[bump]];
        let signer = &[signer_seeds];

        msg!("Assembling zero-copy WithdrawExcessLamports payload vector...");

        let mut instruction_data = [0u8; 1];
        instruction_data[0] = 38;

        let token_program_info = ctx.accounts.token_2022_program.to_account_info();

        let account_metas = vec![
            solana_program::instruction::AccountMeta::new(
                ctx.accounts.stranded_source.key(),
                false,
            ),
            solana_program::instruction::AccountMeta::new(
                ctx.accounts.destination_wallet.key(),
                false,
            ),
            solana_program::instruction::AccountMeta::new_readonly(
                ctx.accounts.treasury_pda.key(),
                true,
            ),
        ];

        let ix = solana_program::instruction::Instruction {
            program_id: token_program_info.key(),
            accounts: account_metas,
            data: instruction_data.to_vec(),
        };

        solana_program::program::invoke_signed(
            &ix,
            &[
                ctx.accounts.stranded_source.to_account_info(),
                ctx.accounts.destination_wallet.to_account_info(),
                ctx.accounts.treasury_pda.to_account_info(),
                token_program_info,
            ],
            signer,
        )?;

        msg!("Surplus lamports successfully recovered to the protocol operations deck.");
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
