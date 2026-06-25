use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::Token2022,
    token_interface::{
        mint_to, token_metadata_initialize, Mint, MintTo, TokenAccount, TokenMetadataInitialize,
    },
};

declare_id!("7mKHgy7WVZiZU29HcfovcLzzbcZsLoLDqJNbCksDa11L");

#[program]
pub mod anchor_nft {
    use super::*;

    pub fn mint_nft(
        ctx: Context<MintNft>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        let cpi_accounts = TokenMetadataInitialize {
            token_program_id: ctx.accounts.token_program.to_account_info(),
            metadata: ctx.accounts.nft_mint.to_account_info(),
            update_authority: ctx.accounts.authority.to_account_info(),
            mint: ctx.accounts.nft_mint.to_account_info(),
            mint_authority: ctx.accounts.authority.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);

        // 1. Initialize metadata (Token-2022 expands the account allocation layout size here)
        token_metadata_initialize(cpi_ctx, name, symbol, uri)?;

        // 2. Dynamic Rent Top-Up System
        let mint_account_info = ctx.accounts.nft_mint.to_account_info();
        let rent = Rent::get()?;

        // Read the freshly expanded data length directly from the validator's modified runtime memory state
        let required_lamports = rent.minimum_balance(mint_account_info.data_len());
        let current_lamports = mint_account_info.lamports();

        if required_lamports > current_lamports {
            let diff = required_lamports - current_lamports;

            // Execute an inner system transfer to fund the expanded byte structure
            anchor_lang::system_program::transfer(
                CpiContext::new(
                    ctx.accounts.system_program.to_account_info(),
                    anchor_lang::system_program::Transfer {
                        from: ctx.accounts.authority.to_account_info(),
                        to: mint_account_info.clone(),
                    },
                ),
                diff,
            )?;
        }

        // 3. Mint token to user ATA
        mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.nft_mint.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    authority: ctx.accounts.authority.to_account_info(),
                },
            ),
            1,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(name: String, symbol: String, uri: String)]
pub struct MintNft<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        signer,
        payer = authority,
        mint::decimals = 0,
        mint::authority = authority,
        extensions::metadata_pointer::authority = authority,
        extensions::metadata_pointer::metadata_address = nft_mint
    )]
    pub nft_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = nft_mint,
        associated_token::authority = authority,
        associated_token::token_program = token_program,
    )]
    pub token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
