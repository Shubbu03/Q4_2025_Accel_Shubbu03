use pinocchio::program_error::ProgramError;

pub mod check_contributions;
pub mod contribute;
pub mod initialize;
pub mod refund;

pub use check_contributions::*;
pub use contribute::*;
pub use initialize::*;
pub use refund::*;

#[repr(u8)]
pub enum FundraiserInstruction {
    Initialize,
    Contribute,
    CheckContribution,
    Refund,
}

impl TryFrom<&u8> for FundraiserInstruction {
    type Error = ProgramError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match *value {
            0 => Ok(FundraiserInstruction::Initialize),
            1 => Ok(FundraiserInstruction::Contribute),
            2 => Ok(FundraiserInstruction::CheckContribution),
            3 => Ok(FundraiserInstruction::Refund),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
