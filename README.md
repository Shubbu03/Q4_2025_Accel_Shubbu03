# 🚀 Turbin3 Accelerated Builder Program

This repository documents my journey through the Turbin3 Accelerated Builder Program, showcasing practical implementations and challenges completed while building production-grade Solana programs.

## 📁 Repository Structure

The repository is organized by weeks, with each week focusing on specific Solana development concepts and techniques:

```
Q4_2025_Accel_Shubbu03/
├── week-01-tkn-ext-litesvm/          # Token Extensions & LiteSVM
│   ├── whitelist-transfer-hook/      # Transfer hook implementation
│   ├── escrow-litesvm/               # Escrow with time-lock & LiteSVM tests
│   └── week-1-challenge/             # Vault with transfer hook functionality
├── week-02-ers-indexing/             # Ephemeral Rollups & Indexing
│   ├── magicblock-er/                # MagicBlock ER state management with VRF
│   └── magicblock-er-ai-agent/       # AI-powered wallet analysis with Solana GPT Oracle
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

### 3️⃣ Vault + Transfer Hook Integration (Challenge)

A production-grade vault system integrated with SPL Token 2022 extensions, demonstrating the complete lifecycle of whitelisted token custody with emergency controls.

**🎯 Challenge Objectives:**
- Build a vault program that integrates with a custom transfer hook
- Implement Token 2022 extensions (Transfer Hook + Permanent Delegate)
- Create comprehensive LiteSVM test suite

**Key Features:**

**Vault Program:**
- `initialize_vault` - Creates mint with transfer hook and permanent delegate extensions
- `deposit` - Users deposit tokens, tracked via per-user Amount PDA
- `withdraw` - Users withdraw tokens, auto-closes Amount PDA when fully withdrawn
- `mint_token` - Admin mints tokens to user ATAs
- `admin_transfer` - Admin force-transfers using permanent delegate (no user signature required)

**Whitelist Transfer Hook:**
- Per-user whitelist PDAs (`seeds = [b"whitelist", mint, user]`)
- Runtime transfer validation via `transfer_hook` instruction
- Admin controls: `add_to_whitelist`, `remove_from_whitelist`
- Extra Account Meta List for automatic account resolution

**Architecture Highlights:**
- VaultConfig PDA stores admin, vault ATA, mint, and bump
- Amount PDA per user tracks deposited balance
- Transfer hook validates both source and destination whitelists on every transfer
- Permanent delegate enables admin emergency interventions

**Test Coverage:**
- Vault initialization with hook configuration
- Deposit/withdraw flows with whitelist validation
- Multiple deposits accumulation
- Partial withdrawals
- Whitelist enforcement (should fail without whitelist)
- Emergency transfers using permanent delegate

**Tech Stack:** Anchor, SPL Token 2022, LiteSVM, Transfer Hook Interface, Permanent Delegate Extension

**Program IDs:**
- Vault: `3NiEScNK9VCHsUKbaGQVJTef5iPTQuRt4jBkWxkDXMfu`
- Transfer Hook: `2Bc7QG4A4sxTsEhefSRBQRVuWcgJvHA5jd4FcKZ5TDxm`

[📂 View Vault](./week-01-tkn-ext-litesvm/week-1-challenge/vault) | [📂 View Hook](./week-01-tkn-ext-litesvm/week-1-challenge/whitelist-transfer-hook)

---

## Week #2: Ephemeral Rollups + RPCs and Indexing

This week explored MagicBlock's Ephemeral Rollups for high-throughput state management and VRF for verifiable randomness on Solana.

### 1️⃣ MagicBlock Ephemeral Rollups State Account

A PDA-based user account system demonstrating delegation to ephemeral rollups for fast, low-cost state mutations with VRF integration.

**🎯 Core Concepts:**
- **Ephemeral Rollups**: Off-chain execution layer that commits state back to Solana base layer
- **Delegate/Undelegate Pattern**: Move accounts between base layer and ephemeral rollup
- **VRF Integration**: Switchboard-powered verifiable randomness for unpredictable state updates
- **Two-Layer Architecture**: Fast mutations on ephemeral layer, finality on base layer

**Key Features:**

**State Management:**
- `initialize` - Creates PDA user account on base layer (`seeds = [b"user", user_pubkey]`)
- `delegate` - Moves account to ephemeral rollup for high-speed mutations
- `undelegate` - Commits final state and returns account to base layer
- `update` - VRF callback that updates state with verifiable randomness
- `update_commit` - Updates state on ephemeral rollup with auto-commit to base layer
- `close` - Closes account and reclaims rent

**UserAccount State:**
```rust
pub struct UserAccount {
    pub user: Pubkey,   // Owner
    pub data: u64,      // Mutable state
    pub bump: u8,       // PDA bump seed
}
```

**Architecture Flow:**
```
Base Layer (Solana Devnet)
    ↓ delegate
Ephemeral Rollup (MagicBlock)
    ↓ high-speed mutations (update_commit)
    ↓ VRF randomness requests
    ↓ commit
Base Layer (state persisted)
```

**VRF Integration:**
- Uses Switchboard oracle queues for randomness requests
- `request_randomness` instruction with caller seed
- Callback to `update` instruction with 32-byte random value
- Enables unpredictable state changes (gaming, lotteries, random selection)

**Ephemeral Rollups Benefits:**
- **10-100x cheaper** transactions on ephemeral layer
- **Sub-second finality** for state mutations
- **Automatic commitment** back to base layer for security
- **Validator delegation** for custom execution environments

**Tech Stack:** Anchor, MagicBlock Ephemeral Rollups SDK, Magicblock VRF

**Program ID:** `9ChqoFDgVmmvD6Hcajv2JppVZ7S1qPDozrTw4V7q2yLP`

**Endpoints:**
- Base Layer: `https://api.devnet.solana.com`
- Ephemeral Rollup: `https://devnet.magicblock.app/`

[📂 View Project](./week-02-ers-indexing/magicblock-er)

---

### 2️⃣ MagicBlock ER AI Agent

A full-stack AI-powered wallet analysis system that leverages Solana GPT Oracle for intelligent transaction pattern analysis with a modern Next.js frontend.

**🎯 Core Concepts:**
- **AI Integration**: Solana GPT Oracle for LLM-powered wallet analysis
- **Asynchronous Processing**: CPI calls to oracle with callback pattern
- **Full-Stack Architecture**: Rust program + Next.js frontend
- **Real-time Analysis**: Polling mechanism for AI response handling

**Key Features:**

**Solana Program:**
- `initialize` - Sets up AI agent with LLM context reference
- `analyze_user` - Triggers AI analysis via CPI to Solana GPT Oracle
- `callback_from_agent` - Handles AI responses and stores analysis results
- `get_analysis` - Retrieves stored analysis for frontend consumption

**Frontend Client:**
- **Next.js 15** with TypeScript and Tailwind CSS
- **Wallet Integration** via Solana wallet adapter
- **Real-time UI** with loading states and error handling
- **Public Key Validation** and input sanitization
- **Analysis Polling** for async AI response handling

**AI Analysis Flow:**
```
User Input (Public Key)
    ↓
Frontend Validation
    ↓
Program: analyze_user()
    ↓
CPI to Solana GPT Oracle
    ↓
AI Processing (Async)
    ↓
callback_from_agent()
    ↓
Store Analysis Result
    ↓
Frontend Polling
    ↓
Display Results
```

**Account Structure:**
- **Agent Account** - Stores LLM context reference and configuration
- **Analysis Result** - Stores AI analysis with user, analysis text, timestamp
- **Oracle Integration** - Uses Solana GPT Oracle for AI processing

**State Management:**
```rust
pub struct Agent {
    pub context: Pubkey,  // LLM context reference
    pub bump: u8,
}

pub struct AnalysisResult {
    pub user: Pubkey,
    pub analysis: String,  // Max 500 chars
    pub timestamp: i64,
    pub bump: u8,
}
```

**Frontend Features:**
- Wallet connection and management
- Public key input with validation
- Real-time analysis status updates
- Modern, responsive dark mode UI
- Error handling and user feedback
- TypeScript type safety throughout

**Tech Stack:** Anchor, Solana GPT Oracle, Next.js 15, TypeScript, Tailwind CSS, Solana Wallet Adapter

**Program ID:** `53GFYSJPbrYcaqD3o54z5WCWcCM8WGqixgUjc4nsw2tY`

[📂 View Project](./week-02-ers-indexing/magicblock-er-ai-agent)

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
