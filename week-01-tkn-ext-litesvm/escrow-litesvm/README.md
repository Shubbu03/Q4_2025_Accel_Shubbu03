# Escrow LiteSVM

A Solana escrow program with time-lock functionality, built with Anchor and tested using LiteSVM.

## Overview

This escrow program enables trustless token swaps between two parties with an optional time-lock mechanism. A maker deposits tokens and specifies the terms, and a taker can accept the offer only after a specified time period has elapsed.

**Program ID**: `7Brfv9ixTj71Nvt8kbQJRj4RWw71y6cwyzSVMKFZzYr9`

## Features

### Core Functionality
- **Make**: Create an escrow offer by depositing Token A and specifying the amount of Token B to receive
- **Take**: Accept an escrow offer by providing Token B and receiving Token A (time-lock protected)
- **Refund**: Cancel the escrow and retrieve deposited tokens (maker only)

### Time-Lock Protection
The program implements a Unix timestamp-based time-lock that prevents the taker from accepting an offer before a specified time. This is useful for:
- Vesting schedules
- Delayed OTC trades
- Fair launch mechanics
- Cooldown periods

**Security Note**: The time-lock value is stored in the escrow account state and cannot be manipulated by the taker. All validation happens on-chain using `Clock::get()?.unix_timestamp`.

## Architecture

### Program Structure

```
programs/escrow-litesvm/src/
├── lib.rs                      # Program entry points
├── error.rs                    # Custom error definitions
├── state/
│   ├── mod.rs
│   └── escrow.rs              # Escrow account structure
├── instructions/
│   ├── mod.rs
│   ├── make.rs                # Create escrow logic
│   ├── take.rs                # Accept escrow logic (time-lock enforced)
│   └── refund.rs              # Cancel escrow logic
└── tests/
    └── mod.rs                 # LiteSVM integration tests
```

### State: Escrow Account

```rust
pub struct Escrow {
    pub seed: u64,                      // Unique identifier
    pub maker: Pubkey,                  // Creator of the escrow
    pub mint_a: Pubkey,                 // Token being deposited
    pub mint_b: Pubkey,                 // Token being requested
    pub receive: u64,                   // Amount of Token B to receive
    pub min_accept_lockin_time: i64,    // Unix timestamp - earliest take time
    pub bump: u8,                       // PDA bump seed
}
```

### Instructions

#### 1. Make
Creates an escrow offer and deposits tokens into a vault.

**Parameters**:
- `seed: u64` - Unique identifier for this escrow
- `deposit: u64` - Amount of Token A to deposit
- `receive: u64` - Amount of Token B requested
- `min_accept_lockin_time: i64` - Unix timestamp when taker can accept (0 for immediate)

**Accounts**:
- Maker's Token A account (debited)
- Vault (PDA, created and funded)
- Escrow state account (PDA, created)

#### 2. Take
Accepts an escrow offer after the time-lock has elapsed.

**Time-Lock Validation**: 
```rust
require!(
    Clock::get()?.unix_timestamp >= escrow.min_accept_lockin_time,
    EscrowError::TakeOfferTimeNotElapsed
);
```

**Accounts**:
- Taker's Token B account (debited)
- Taker's Token A account (credited)
- Maker's Token B account (credited)
- Vault (closed, rent refunded to maker)
- Escrow state account (closed, rent refunded to maker)

#### 3. Refund
Allows the maker to cancel the offer and retrieve their tokens.

**Accounts**:
- Maker's Token A account (credited)
- Vault (closed)
- Escrow state account (closed)

## Setup

### Prerequisites
- Rust 1.75+
- Solana CLI 1.18+
- Anchor 0.30+
- Yarn

### Installation

```bash
# Clone the repository
git clone <repo-url>
cd escrow-litesvm

# Install dependencies
yarn install

# Build the program
anchor build
# or
cargo build-sbf
```

## Testing

The project uses LiteSVM for fast, lightweight testing without requiring a local validator.

### Run Tests

```bash
# Run all tests
cargo test-sbf

# Run with verbose output
cargo test-sbf -- --nocapture

# Run specific test
cargo test-sbf test_take_with_time_lock_fails_before_unlock
```

### Test Coverage

- ✅ `test_make` - Escrow creation and state validation
- ✅ `test_take` - Successful token swap (no time-lock)
- ✅ `test_refund` - Maker cancels escrow
- ✅ `test_take_with_time_lock_fails_before_unlock` - Rejects early take attempts
- ✅ `test_take_with_time_lock_succeeds_after_unlock` - Allows take after time elapsed

## Usage Example

### TypeScript/JavaScript (Anchor)

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { EscrowLitesvm } from "../target/types/escrow_litesvm";

// Initialize
const program = anchor.workspace.EscrowLitesvm as Program<EscrowLitesvm>;
const provider = anchor.AnchorProvider.env();

// Create escrow with 1-hour time lock
const seed = new anchor.BN(Date.now());
const deposit = new anchor.BN(1_000_000); // 1 Token A (6 decimals)
const receive = new anchor.BN(500_000);   // 0.5 Token B (6 decimals)
const timelock = new anchor.BN(Math.floor(Date.now() / 1000) + 3600); // +1 hour

const [escrow] = anchor.web3.PublicKey.findProgramAddressSync(
  [
    Buffer.from("escrow"),
    maker.publicKey.toBuffer(),
    seed.toArrayLike(Buffer, "le", 8),
  ],
  program.programId
);

await program.methods
  .make(seed, deposit, receive, timelock)
  .accounts({
    maker: maker.publicKey,
    mintA: mintA,
    mintB: mintB,
    // ... other accounts
  })
  .signers([maker])
  .rpc();

// Take escrow (after timelock)
await program.methods
  .take()
  .accounts({
    taker: taker.publicKey,
    maker: maker.publicKey,
    // ... other accounts
  })
  .signers([taker])
  .rpc();
```

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 6000 | `TakeOfferTimeNotElapsed` | Attempted to take escrow before time-lock expired |

## Development Workflow

```bash
# 1. Make changes to the program
vim programs/escrow-litesvm/src/instructions/take.rs

# 2. Rebuild
cargo build-sbf

# 3. Run tests
cargo test-sbf

# 4. Deploy (localnet)
anchor deploy

# 5. Deploy (devnet)
anchor deploy --provider.cluster devnet
```

## LiteSVM Advantages

Unlike traditional Solana testing with `solana-test-validator`:

- ⚡ **10x faster** test execution
- 💾 **Lower memory footprint** (no validator process)
- 🔧 **Simpler setup** (no background processes)
- 🎯 **Deterministic** (consistent state per test)

## Contributing

Contributions welcome. Please ensure all tests pass before submitting PRs:

```bash
cargo test-sbf
cargo fmt
cargo clippy -- -D warnings
```