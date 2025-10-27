use pinocchio::program_error::ProgramError;

#[derive(Clone, PartialEq, shank::ShankType)]
pub enum FundraiserError {
    InvalidInstructionData,
    PdaMismatch,
    InvalidOwner,
    InvalidAmount,
    MintMismatch,
    ContributionTooSmall,
    ContributionTooBig,
    FundraiserEnded,
    MaximumContributionsReached,
    TargetNotMet,
    FundraiserNotEnded,
    TargetMet,
}

impl From<FundraiserError> for ProgramError {
    fn from(e: FundraiserError) -> Self {
        Self::Custom(e as u32)
    }
}
