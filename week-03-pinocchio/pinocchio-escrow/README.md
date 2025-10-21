# Pinocchio Escrow Program

A Solana program built with Pinocchio framework that implements a secure token escrow system for atomic token swaps.

## Overview

This escrow program enables trustless token exchanges between two parties. A maker deposits tokens they want to trade, and a taker can complete the swap by providing the requested tokens. The escrow ensures atomicity - either the entire swap succeeds or fails completely.

## Features

- **Make Escrow**: Create a new escrow with tokens you want to trade
- **Take Escrow**: Complete an existing escrow by providing the requested tokens
- **Cancel Escrow**: Cancel your own escrow and reclaim deposited tokens
- **PDA-based Security**: Uses Program Derived Addresses for secure account management
- **Atomic Swaps**: Ensures complete token exchanges or complete failure

## Program Architecture

### State Structure
```rust
pub struct Escrow {
    maker: [u8; 32],           // Public key of escrow creator
    mint_a: [u8; 32],         // Token mint being offered
    mint_b: [u8; 32],         // Token mint being requested
    amount_to_receive: [u8; 8], // Amount of mint_b to receive
    amount_to_give: [u8; 8],    // Amount of mint_a to give
    bump: u8,                  // PDA bump seed
}
```

### Instructions

1. **Make (0)**: Create a new escrow
   - Deposits `amount_to_give` of `mint_a` tokens
   - Requests `amount_to_receive` of `mint_b` tokens
   - Creates escrow PDA account

2. **Take (1)**: Complete an escrow
   - Transfers escrow tokens to taker
   - Transfers requested tokens from taker to maker
   - Closes escrow account

3. **Cancel (2)**: Cancel an escrow
   - Returns deposited tokens to maker
   - Closes escrow account

## Technical Details

- **Framework**: Pinocchio 0.9.2
- **Program ID**: `4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT`
- **Account Size**: 81 bytes (32+32+32+8+8+1)
- **PDA Seeds**: `["escrow", maker_pubkey, bump]`

## Usage

### Creating an Escrow
```rust
// Instruction data: [bump, amount_to_receive (8 bytes), amount_to_give (8 bytes)]
// Accounts: maker, mint_a, mint_b, escrow_account, maker_ata, escrow_ata, 
//           system_program, token_program, associated_token_program, rent_sysvar
```

### Taking an Escrow
```rust
// Instruction data: [bump]
// Accounts: taker, maker, mint_a, mint_b, escrow_account, taker_ata_a, 
//           taker_ata_b, maker_ata_b, escrow_ata, token_program, 
//           associated_token_program, rent_sysvar
```

### Canceling an Escrow
```rust
// Instruction data: [bump]
// Accounts: maker, mint_a, mint_b, escrow_account, maker_ata, escrow_ata,
//           token_program, associated_token_program, rent_sysvar
```

## Security Features

- **Ownership Validation**: All token accounts are validated for correct ownership
- **Mint Validation**: Token accounts are validated against expected mints
- **PDA Verification**: Escrow accounts are verified using derived addresses
- **State Validation**: Escrow state is validated before operations
- **Atomic Operations**: All transfers succeed or fail together

## Development

### Building
```bash
cargo build-sbf
```

### Testing
The project uses **LiteSVM** for testing - a lightweight Solana Virtual Machine implementation that allows for fast, deterministic testing without requiring a full Solana cluster.

```bash
cargo test
```

**Test Features:**
- Uses `litesvm` and `litesvm-token` for VM simulation
- Tests complete instruction flows (Make, Take, Cancel)
- Validates PDA derivation and account creation
- Verifies token transfers and account state changes
- Includes compute unit consumption tracking

### Dependencies
- `pinocchio`: Core framework
- `pinocchio-token`: Token operations
- `pinocchio-system`: System operations
- `pinocchio-associated-token-account`: ATA management
- `litesvm`: Testing framework
- `litesvm-token`: Token testing utilities

