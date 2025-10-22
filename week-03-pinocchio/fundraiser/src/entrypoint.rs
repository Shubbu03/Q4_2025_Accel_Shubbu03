#![allow(unexpected_cfgs)]

use crate::instructions::{self, FundraiserInstruction};
use pinocchio::{
    account_info::AccountInfo, default_panic_handler, no_allocator, program_entrypoint,
    program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

// This is the entrypoint for the program.
program_entrypoint!(process_instruction);
//Do not allocate memory.
no_allocator!();
// Use the no_std panic handler.
default_panic_handler!();

#[inline(always)]
fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (ix_disc, instruction_data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match FundraiserInstruction::try_from(ix_disc)? {
        FundraiserInstruction::Initialize => instructions::initialize(accounts, instruction_data),
        FundraiserInstruction::Contribute => instructions::contribute(accounts, instruction_data),
        FundraiserInstruction::CheckContribution => {
            instructions::check_contributions(accounts, instruction_data)
        }
        FundraiserInstruction::Refund => {
            instructions::refund_to_contributors(accounts, instruction_data)
        }
    }
}
