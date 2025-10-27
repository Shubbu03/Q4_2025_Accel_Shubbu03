#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;

#[cfg(feature = "std")]
extern crate std;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod states;

#[cfg(all(test, feature = "std"))]
pub mod tests;

pinocchio_pubkey::declare_id!("9vKWS1DteTPdRFPRzaJgSYwUsNV8VQDcdeiz5WRvavdv");
