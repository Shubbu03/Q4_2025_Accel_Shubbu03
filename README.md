# 🚀 Turbin3 Accelerated Builder Program

This repository documents my journey through the Turbin3 Accelerated Builder Program, showcasing practical implementations and challenges completed while building production-grade Solana programs.

## 📁 Repository Structure

The repository is organized by weeks, with each week focusing on specific Solana development concepts and techniques:

```
Q4_2025_Accel_Shubbu03/
├── week-01-tkn-ext-litesvm/          # Token Extensions & LiteSVM
│   ├── whitelist-transfer-hook/      # Transfer hook implementation
│   └── escrow-litesvm/               # Escrow with time-lock & LiteSVM tests
└── README.md
```

---

## Week #1: Token Extensions & LiteSVM

This week focused on SPL Token 2022 extensions and LiteSVM for fast, efficient testing. I built two production-ready programs demonstrating advanced Solana concepts.

### 1️⃣ Whitelist Transfer Hook

A SPL Token 2022 transfer hook that enforces whitelist restrictions on token transfers using per-user PDAs.

**🎯 Challenges Completed:**
- ✅ **Make the whitelist a PDA per user instead of a Vec**: Implemented a scalable per-user PDA architecture using `seeds = [b"whitelist", mint, user]` for efficient, rent-optimized access control (no vector reallocations)
- ✅ **Initialize the mint with the transfer hook in the program**: Moved mint initialization logic from client-side to on-chain, ensuring the transfer hook is configured programmatically during mint creation

**Key Features:**
- Per-user whitelist PDAs for minimal account size and no reallocations
- Automatic transfer validation via SPL Token 2022 hooks
- Extra Account Meta List for seamless runtime account resolution
- Admin controls: `add_to_whitelist` and `remove_from_whitelist`

**Tech Stack:** Anchor, SPL Token 2022, Transfer Hook Interface

[📂 View Project](./week-01-tkn-ext-litesvm/whitelist-transfer-hook)

---

### 2️⃣ Escrow LiteSVM

A trustless token swap escrow program with Unix timestamp-based time-lock functionality, tested using LiteSVM for 10x faster execution.

**🎯 Challenges Completed:**
- ✅ **Completed tests for take and refund instructions**: Implemented comprehensive test coverage for both happy paths and edge cases
- ✅ **Add a time lock functionality and update tests accordingly**: Built a time-lock mechanism using `Clock::get()?.unix_timestamp` with custom error handling (`TakeOfferTimeNotElapsed`) and added test cases for pre/post-unlock scenarios

**Key Features:**
- Three core instructions: `make`, `take`, `refund`
- Time-lock protection prevents premature escrow acceptance
- Vault PDA for secure token custody
- Escrow state tracks maker, tokens, amounts, and unlock time
- LiteSVM testing: deterministic, fast, low-memory

**Test Coverage:**
- `test_make` - Escrow creation and state validation
- `test_take` - Successful token swap
- `test_refund` - Maker cancellation
- `test_take_with_time_lock_fails_before_unlock` - Rejects early attempts
- `test_take_with_time_lock_succeeds_after_unlock` - Allows post-unlock swap

**Tech Stack:** Anchor, LiteSVM, SPL Token

**Program ID:** `7Brfv9ixTj71Nvt8kbQJRj4RWw71y6cwyzSVMKFZzYr9`

[📂 View Project](./week-01-tkn-ext-litesvm/escrow-litesvm)

---

## Week #2: Ephemeral Rollups + RPCs and Indexing

Soon..

---

## Week #3: All about Pinocchio

Soon..

---

## Week #4: MPL + Codama + Group

Soon..

---

## 📬 Contact

**You can reach me here**  
✉️ [thatcoderguyshubham@gmail.com](mailto:thatcoderguyshubham@gmail.com)  
𝕏  [@blackbaloon03](https://x.com/blackbaloon03)
