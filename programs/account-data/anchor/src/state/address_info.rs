use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct AddressInfo {
    #[max_len(50)]
    pub name: String,
    pub house_number: u8,
    #[max_len(50)]
    pub street: String,
    #[max_len(50)]
    pub city: String,
}
