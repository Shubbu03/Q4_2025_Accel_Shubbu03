# Ephemeral Rollups State Account

Solana Anchor program demonstrating **MagicBlock Ephemeral Rollups** integration with VRF (Verifiable Random Function) support. Shows how to delegate account state to ephemeral rollups for fast, low-cost mutations that commit back to the base layer.

## Architecture

This program implements a PDA-based user account system that can:
- **Delegate** state to ephemeral rollups for high-throughput operations
- **Commit** changes back to Solana's base layer
- **Request randomness** via VRF for unpredictable state updates
- Maintain state consistency across layers

### Core Flow

```
Base Layer (Solana Devnet)
    ↓ delegate
Ephemeral Rollup (MagicBlock)
    ↓ high-speed mutations
    ↓ commit
Base Layer (state persisted)
```

## Program Structure

```
programs/er-state-account/src/
├── lib.rs                    # Program entrypoint
├── state/
│   ├── mod.rs
│   └── user_account.rs       # UserAccount state (Pubkey + u64 data)
└── instructions/
    ├── init_user.rs          # Initialize PDA account
    ├── update_user.rs        # Update with VRF randomness
    ├── update_commit.rs      # Update and commit to base layer
    ├── delegate.rs           # Delegate to ephemeral rollup
    ├── undelegate.rs         # Undelegate back to base layer
    ├── request_randomness.rs # VRF randomness request
    └── close_user.rs         # Close account, reclaim rent
```

### State Account

```rust
pub struct UserAccount {
    pub user: Pubkey,   // Owner
    pub data: u64,      // Mutable state
    pub bump: u8,       // PDA bump seed
}
```

PDA seeds: `["user", user_pubkey]`


## Setup

```bash
# Install dependencies
yarn install

# Build program
anchor build

# Run tests (requires devnet SOL)
anchor test --skip-local-validator
```

## Instructions

### 1. Initialize
Creates PDA user account on base layer.

```typescript
await program.methods.initialize()
  .accountsPartial({
    user: wallet.publicKey,
    userAccount: userAccountPDA,
  })
  .rpc();
```

### 2. Delegate
Moves account to ephemeral rollup for fast mutations.

```typescript
await program.methods.delegate()
  .accountsPartial({
    user: wallet.publicKey,
    userAccount: userAccountPDA,
    validator: MAGICBLOCK_VALIDATOR,
  })
  .rpc();
```

### 3. Update Commit
Updates state on ephemeral rollup with automatic commit to base layer.

```typescript
await program.methods.updateCommit(new BN(43))
  .accountsPartial({
    user: wallet.publicKey,
    userAccount: userAccountPDA,
  })
  .rpc();
```

### 4. Undelegate
Commits final state and returns account to base layer.

```typescript
await program.methods.undelegate()
  .accounts({ user: wallet.publicKey })
  .rpc();
```

### 5. Request Randomness
Requests VRF randomness with callback to `update` instruction.

```typescript
const callerSeed = Buffer.from(crypto.randomBytes(32));
await program.methods.requestRandomness(callerSeed)
  .accountsPartial({
    user: wallet.publicKey,
    userAccount: userAccountPDA,
    oracleQueue: DEFAULT_QUEUE,
  })
  .rpc();
```

### 6. Close
Closes account and returns rent to user.

```typescript
await program.methods.close()
  .accountsPartial({
    user: wallet.publicKey,
    userAccount: userAccountPDA,
  })
  .rpc();
```

## Resources

- [MagicBlock Docs](https://docs.magicblock.gg/)
- [Ephemeral Rollups SDK](https://github.com/magicblock-labs/ephemeral-rollups-sdk)
- [Anchor Documentation](https://www.anchor-lang.com/)