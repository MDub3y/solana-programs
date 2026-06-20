use anchor_lang::prelude::*;

declare_id!("ChT1pY9D9Db9jG7FmG7FmG7FmG7FmG7FmG7FmG7FmG7F");

#[program]
pub mod counter_anchor {
    use super::*;

    pub fn increment(ctx: Context<Increment>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.count += 1;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Increment<'info> {
    #[account(mut)]
    pub counter: Account<'info, Counter>,
}

#[account]
pub struct Counter {
    pub count: u64,
}
