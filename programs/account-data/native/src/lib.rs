use solana_program::entrypoint;

use crate::processor::processs_instruction;

pub mod instructions;
pub mod processor;
pub mod state;

entrypoint!(processs_instruction);
