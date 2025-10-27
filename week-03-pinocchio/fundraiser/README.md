# Fundraiser Program

A Solana program written with Pinocchio (no_std) for managing token-based fundraisers. This program allows users to create fundraisers and collect contributions with built-in safety mechanisms including refund capabilities.

## Program ID

```
9vKWS1DteTPdRFPRzaJgSYwUsNV8VQDcdeiz5WRvavdv
```

## Overview

The fundraiser program enables decentralized fundraising where:
- A **maker** creates a fundraiser with a target amount and duration
- **Contributors** can contribute tokens within specified limits
- The maker can withdraw funds once the target is met
- Contributors can receive refunds if the target isn't met before the fundraiser ends

## Instructions

### 1. Initialize

Creates a new fundraiser with specified parameters.

**Accounts:**
- `maker` (signer) - Creator of the fundraiser
- `mint_to_raise` - Token mint for fundraising
- `fundraiser` (PDA) - Fundraiser state account
- `vault` (ATA) - Token account holding contributions
- `sysvar_rent` - Rent sysvar
- `token_program` - SPL Token program
- `system_program` - System program
- `associated_token_program` - Associated Token Account program

**Instruction Data:**
- `amount` (u64) - Target amount to raise
- `duration` (u8) - Fundraiser duration in days
- `bump` (u8) - PDA bump seed

**Constraints:**
- Minimum amount to raise: 3 tokens
- Fundraiser must be uninitialized
- Mint must be a valid SPL token mint

### 2. Contribute

Allows contributors to send tokens to the fundraiser vault.

**Accounts:**
- `contributor` (signer) - Contributor wallet
- `mint_to_raise` - Token mint for fundraising
- `fundraiser` - Fundraiser state account
- `contributor_account` (PDA) - Contributor state account
- `contributor_ata` (ATA) - Contributor's token account
- `vault` (ATA) - Fundraiser's token vault
- `sysvar_rent` - Rent sysvar
- `token_program` - SPL Token program
- `system_program` - System program

**Instruction Data:**
- `amount` (u64) - Amount to contribute
- `bump` (u8) - Contributor PDA bump seed

**Constraints:**
- Contribution must be more than 1 native unit (not dust)
- Maximum contribution: 10% of target amount
- Total contributor contributions cannot exceed max per-contributor limit
- Fundraiser must not have ended
- Mint must match fundraiser's mint

**Behavior:**
- Creates contributor account if first contribution
- Transfers tokens from contributor to vault
- Updates contributor and fundraiser state

### 3. CheckContribution

Allows the maker to check if target is met and withdraw funds.

**Accounts:**
- `maker` (signer) - Fundraiser creator
- `mint_to_raise` - Token mint
- `fundraiser` - Fundraiser state account
- `vault` (ATA) - Fundraiser's token vault
- `maker_ata` (ATA) - Maker's token account
- `sysvar_rent` - Rent sysvar
- `token_program` - SPL Token program
- `system_program` - System program
- `associated_token_program` - Associated Token Account program

**Constraints:**
- Maker must be signer
- Target amount must be met
- Creates maker ATA if needed
- Transfers all vault tokens to maker

**Behavior:**
- Validates that vault amount ≥ target amount
- Transfers all funds from vault to maker
- Only executable once target is met

### 4. Refund

Allows contributors to reclaim their contributions if target isn't met.

**Accounts:**
- `contributor` (signer) - Contributor wallet
- `maker` - Fundraiser creator
- `mint_to_raise` - Token mint
- `fundraiser` - Fundraiser state account
- `contributor_account` (PDA) - Contributor state account
- `contributor_ata` (ATA) - Contributor's token account
- `vault` (ATA) - Fundraiser's token vault
- `token_program` - SPL Token program
- `system_program` - System program

**Constraints:**
- Fundraiser duration must have ended
- Target must NOT be met (vault < target)
- Contributor must have made a contribution
- Reduces fundraiser's current_amount by refunded amount

**Behavior:**
- Transfers contributor's contribution back from vault
- Updates fundraiser state
- Does not close contributor account (for future use)

## State Accounts

### Fundraiser

Tracks fundraiser state. Derives from:
```
PDA: [b"fundraiser", maker_pubkey, bump]
```

**Data Structure:**
- `maker` (Pubkey) - Creator
- `mint_to_raise` (Pubkey) - Token mint
- `amount_to_raise` (u64) - Target amount
- `current_amount` (u64) - Current amount raised
- `time_started` (i64) - Unix timestamp
- `duration` (u8) - Duration in days
- `bump` (u8) - PDA bump

### Contributor

Tracks individual contributor's total contribution. Derives from:
```
PDA: [b"contributor", fundraiser_pubkey, contributor_pubkey, bump]
```

**Data Structure:**
- `amount` (u64) - Total contributed

## Constants

- `MIN_AMOUNT_TO_RAISE`: 3 - Minimum tokens to raise
- `SECONDS_TO_DAYS`: 86400 - Seconds in a day
- `MAX_CONTRIBUTION_PERCENTAGE`: 10 - Max contribution (% of target)
- `PERCENTAGE_SCALER`: 100 - Percentage scaler

## Errors

- `InvalidInstructionData` - Invalid instruction data
- `PdaMismatch` - PDA validation failed
- `InvalidOwner` - Invalid account owner
- `InvalidAmount` - Invalid amount specified
- `MintMismatch` - Mint mismatch
- `ContributionTooSmall` - Contribution below minimum
- `ContributionTooBig` - Contribution above maximum (10%)
- `FundraiserEnded` - Fundraiser duration expired
- `MaximumContributionsReached` - Contributor hit limit
- `TargetNotMet` - Target amount not reached
- `FundraiserNotEnded` - Fundraiser still active
- `TargetMet` - Target already achieved

## Testing

Tests are written using **litesvm** for fast, unit-test style Solana program testing. The test suite is located in `src/tests/` and covers all program functionality including:

- Fundraiser initialization
- Contribution mechanics
- Maximum contribution limits
- Time-based constraints
- Withdrawal validation
- Refund scenarios

## Dependencies

- `pinocchio` - no_std Solana framework
- `bytemuck` - Zero-copy serde
- `pinocchio-associated-token-account` - ATA management
- `pinocchio-token` - SPL Token integration
- `pinocchio-system` - System program integration
- `shank` - Error type generation

