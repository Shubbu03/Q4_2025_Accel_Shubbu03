# Whitelist Transfer Hook

This example demonstrates how to implement a transfer hook using the SPL Token 2022 Transfer Hook interface to enforce whitelist restrictions on token transfers.

Only addresses that have a per-user whitelist entry will be able to transfer tokens for a given mint. The transfer hook requires that the caller supplies the user’s whitelist entry PDA as an extra account; if it does not exist, the transfer fails during account resolution.

---

## Architecture

Instead of a single vector-based whitelist account, this program uses per-user PDAs:

- A `WhitelistEntry` PDA per (mint, user)

State:

```rust
#[account]
#[derive(InitSpace)]
pub struct WhitelistEntry {
    pub bump: u8,
}
```

- Seed derivation: `[b"whitelist", mint.key().as_ref(), user.as_ref()]`
- One PDA per user keeps account sizes minimal and avoids reallocations.

---

## Whitelist management (admin)

Add a user to a mint’s whitelist by creating their PDA; remove by closing it.

```rust
#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct AddToWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = admin,
        space = 8 + WhitelistEntry::INIT_SPACE,
        seeds = [b"whitelist", mint.key().as_ref(), user.as_ref()],
        bump,
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,

    pub system_program: Program<'info, System>,
}

impl<'info> AddToWhitelist<'info> {
    pub fn add_to_whitelist(&mut self, _user: Pubkey, bumps: AddToWhitelistBumps) -> Result<()> {
        self.whitelist_entry.set_inner(WhitelistEntry {
            bump: bumps.whitelist_entry,
        });
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct RemoveFromWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        close = admin,
        seeds = [b"whitelist", mint.key().as_ref(), user.as_ref()],
        bump = whitelist_entry.bump,
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,
}

impl<'info> RemoveFromWhitelist<'info> {
    pub fn remove_from_whitelist(&mut self, _user: Pubkey) -> Result<()> {
        // Closed via `close = admin`
        Ok(())
    }
}
```

- **Add**: initializes the user’s PDA if missing, storing the bump.
- **Remove**: closes the PDA, refunding lamports to `admin`.

---

## Extra account metadata for transfer hook

The transfer hook needs to know how to derive required extra accounts. We store an `ExtraAccountMetaList` for each mint under `seeds = [b"extra-account-metas", mint]`.

```rust
#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        init,
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump,
        space = ExtraAccountMetaList::size_of(
            InitializeExtraAccountMetaList::extra_account_metas()?.len()
        )?,
        payer = payer
    )]
    pub extra_account_meta_list: AccountInfo<'info>,

    pub mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeExtraAccountMetaList<'info> {
    pub fn extra_account_metas() -> Result<Vec<ExtraAccountMeta>> {
        Ok(vec![ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal { bytes: b"whitelist".to_vec() },
                Seed::AccountKey { index: 1 }, // mint
                Seed::AccountKey { index: 3 }, // owner
            ],
            false, // is_signer
            false, // is_writable
        )?])
    }
}
```

- The seeds above tell the runtime to pass the per-user `whitelist_entry` PDA to the hook during transfers: `whitelist_entry = PDA(["whitelist", mint, owner])`.

---

## Transfer hook validation

The transfer hook validates that it’s running during an SPL Token 2022 transfer and that the caller provided the required extra accounts. Because the `whitelist_entry` PDA is constrained in the account list, the absence of that PDA (i.e., the user is not whitelisted) will fail account resolution.

```rust
#[derive(Accounts)]
pub struct TransferHook<'info> {
    #[account(
        token::mint = mint,
        token::authority = owner,
    )]
    pub source_token: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        token::mint = mint,
    )]
    pub destination_token: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: source token account owner, can be SystemAccount or PDA owned by another program
    pub owner: UncheckedAccount<'info>,

    /// CHECK: ExtraAccountMetaList Account
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    #[account(
        seeds = [b"whitelist", mint.key().as_ref(), owner.key().as_ref()],
        bump = whitelist_entry.bump,
    )]
    pub whitelist_entry: Account<'info, WhitelistEntry>,
}

impl<'info> TransferHook<'info> {
    pub fn transfer_hook(&mut self, _amount: u64) -> Result<()> {
        // Ensure this is called during a token-2022 transfer
        self.check_is_transferring()?;
        Ok(())
    }

    fn check_is_transferring(&mut self) -> Result<()> {
        let source_token_info = self.source_token.to_account_info();
        let mut account_data_ref: RefMut<&mut [u8]> = source_token_info.try_borrow_mut_data()?;
        let mut account = PodStateWithExtensionsMut::<PodAccount>::unpack(*account_data_ref)?;
        let account_extension = account.get_extension_mut::<TransferHookAccount>()?;
        if !bool::from(account_extension.transferring) {
            panic!("TransferHook: Not transferring");
        }
        Ok(())
    }
}
```

- No manual whitelist check is needed; the `whitelist_entry` account constraint enforces presence of the PDA for `(mint, owner)`.

---

## Program entrypoints

```rust
pub fn add_to_whitelist(ctx: Context<AddToWhitelist>, user: Pubkey) -> Result<()>
pub fn remove_from_whitelist(ctx: Context<RemoveFromWhitelist>, user: Pubkey) -> Result<()>
pub fn initialize_transfer_hook(ctx: Context<InitializeExtraAccountMetaList>) -> Result<()>
pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()>
```

- Initialize the mint’s extra account meta list once per mint.
- Add/remove users per mint via their PDAs.
- The hook is invoked automatically by SPL Token 2022 during transfers.

---

This per-user PDA design is simple, rent-efficient, and avoids runtime reallocations while providing robust, mint-scoped access control for Token 2022 transfers.