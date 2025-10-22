#![no_std]

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

#[cfg(feature = "std")]
extern crate std;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod states;

pinocchio_pubkey::declare_id!("9vKWS1DteTPdRFPRzaJgSYwUsNV8VQDcdeiz5WRvavdv");
