use bytemuck;
use pinocchio::program_error::ProgramError;

use crate::errors::FundraiserError;

pub trait DataLen {
    const LEN: usize;
}

/// Safely load account data using bytemuck
#[inline(always)]
pub fn load_acc<T: DataLen + bytemuck::Pod>(bytes: &[u8]) -> Result<&T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    bytemuck::try_from_bytes(bytes).map_err(|_| ProgramError::InvalidAccountData)
}

/// Safely load mutable account data using bytemuck
#[inline(always)]
pub fn load_acc_mut<T: DataLen + bytemuck::Pod>(bytes: &mut [u8]) -> Result<&mut T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    bytemuck::try_from_bytes_mut(bytes).map_err(|_| ProgramError::InvalidAccountData)
}

/// Safely load instruction data using bytemuck
#[inline(always)]
pub fn load_ix_data<T: DataLen + bytemuck::Pod>(bytes: &[u8]) -> Result<&T, ProgramError> {
    if bytes.len() != T::LEN {
        return Err(FundraiserError::InvalidInstructionData.into());
    }
    bytemuck::try_from_bytes(bytes).map_err(|_| FundraiserError::InvalidInstructionData.into())
}

/// Convert data to bytes safely
pub fn to_bytes<T: DataLen + bytemuck::Pod>(data: &T) -> &[u8] {
    bytemuck::bytes_of(data)
}

/// Convert mutable data to bytes safely
pub fn to_mut_bytes<T: DataLen + bytemuck::Pod>(data: &mut T) -> &mut [u8] {
    bytemuck::bytes_of_mut(data)
}
