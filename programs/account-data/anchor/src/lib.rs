use anchor_lang::prelude::*;
use instructions::*;

pub mod constants;
pub mod instructions;
pub mod state;

declare_id!("ChT1pY9D9Db9jG7FmG7FmG7FmG7FmG7FmG7FmG7FmG7F");

#[program]
pub mod account_data_anchor {
    use crate::instructions::CreateAddressInfo;

    use super::*;

    pub fn create_address_info(
        ctx: Context<CreateAddressInfo>,
        name: String,
        house_number: u8,
        street: String,
        city: String,
    ) -> Result<()> {
        create::create_address_info(ctx, name, house_number, street, city)
    }
}
