use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{
        mint_to, token_metadata_initialize, Mint, MintTo, TokenAccount, TokenMetadataInitialize,
    },
};

/* declare_id!("NftAnchor1111111111111111111111111111111111"); */
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

        // 2. Initialize the dynamic metadata fields inside the mint account
        token_metadata_initialize(cpi_ctx, name, symbol, uri)?;

        // 3. Mint exactly 1 token to the user's token account
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

    #[account(mut)]
    pub token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}
