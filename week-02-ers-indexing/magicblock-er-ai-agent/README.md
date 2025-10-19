# MagicBlock ER AI Agent

A Solana-based AI agent that analyzes wallet transaction patterns using the Solana GPT Oracle. This project combines a Rust-based Solana program with a Next.js frontend to provide intelligent wallet analysis capabilities.

## 🏗️ Architecture

### Solana Program (`programs/magicblock-er-ai-agent/`)
- **Language**: Rust with Anchor framework
- **Program ID**: `53GFYSJPbrYcaqD3o54z5WCWcCM8WGqixgUjc4nsw2tY`
- **Oracle Integration**: Uses `solana-gpt-oracle` for AI-powered analysis
- **Key Features**:
  - Initialize AI agent with LLM context
  - Analyze user wallet activity
  - Handle AI callbacks and store results
  - Retrieve analysis results

### Frontend Client (`client/`)
- **Framework**: Next.js 15 with TypeScript
- **UI**: Tailwind CSS with dark mode support
- **Wallet Integration**: Solana wallet adapter
- **Features**:
  - Wallet connection and management
  - Public key input and validation
  - Real-time analysis polling
  - Modern, responsive UI

## 🚀 Features

- **AI-Powered Analysis**: Leverages Solana GPT Oracle for intelligent wallet analysis
- **Real-time Processing**: Asynchronous analysis with polling mechanism
- **Wallet Integration**: Seamless Solana wallet connection
- **Modern UI**: Clean, responsive interface with dark mode
- **Error Handling**: Comprehensive error states and user feedback
- **Type Safety**: Full TypeScript implementation

## 📋 Prerequisites

- Rust (latest stable)
- Node.js 18+ and Yarn
- Solana CLI tools
- Anchor framework
- A Solana wallet (Phantom, Solflare, etc.)

## 🛠️ Installation

### 1. Clone and Setup
```bash
git clone <repository-url>
cd magicblock-er-ai-agent
```

### 2. Install Dependencies
```bash
# Install Rust dependencies
cargo build

# Install client dependencies
cd client
yarn install
```

### 3. Build the Program
```bash
# Build the Solana program
anchor build

# Deploy to localnet (optional)
anchor deploy --provider.cluster localnet
```

## 🏃‍♂️ Running the Application

### Start the Client
```bash
cd client
yarn dev
```

The application will be available at `http://localhost:3000`.

### Start Local Solana Test Validator (Optional)
```bash
solana-test-validator --reset
```

## 📖 Usage

1. **Connect Wallet**: Click the wallet button to connect your Solana wallet
2. **Enter Public Key**: Input any Solana public key you want to analyze
3. **Analyze**: Click "Analyze" to start the AI-powered analysis
4. **View Results**: The analysis will appear once the AI processing is complete

## 🔧 Program Instructions

### `initialize`
Initializes the AI agent with LLM context for analysis capabilities.

### `analyze_user`
Triggers AI analysis of a given user's wallet activity.

### `callback_from_agent`
Handles AI responses and stores analysis results.

### `get_analysis`
Retrieves stored analysis results for a user.

## 🏛️ Account Structure

- **Agent Account**: Stores agent configuration and LLM context reference
- **Analysis Result**: Stores AI analysis results for each user
- **Oracle Integration**: Uses Solana GPT Oracle for AI processing

## 📁 Project Structure

```
magicblock-er-ai-agent/
├── programs/magicblock-er-ai-agent/    # Solana program
│   ├── src/
│   │   ├── instructions/               # Program instructions
│   │   ├── state/                      # Account structures
│   │   └── lib.rs                      # Main program logic
│   └── Cargo.toml
├── client/                             # Next.js frontend
│   ├── src/
│   │   ├── components/                 # React components
│   │   ├── lib/                        # Program integration
│   │   └── types/                      # TypeScript types
│   └── package.json
├── migrations/                         # Anchor migrations
└── tests/                             # Test files
```