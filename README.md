# PrivateMed Solana — Encrypted Medical Records on Arcium

> Patient data verified inside Arcium MXE. On-chain records contain only compliance proofs — never raw PII. Enables trustless interoperability between hospitals without exposing patient records.

[![Solana Devnet](https://img.shields.io/badge/Solana-devnet-9945FF)](https://explorer.solana.com/?cluster=devnet)
[![Arcium MXE](https://img.shields.io/badge/Arcium-MXE%20cluster%20456-00D4FF)](https://arcium.com)
[![Anchor](https://img.shields.io/badge/Anchor-0.32.1-orange)](https://anchor-lang.com)

---

## Problem

Medical records in traditional systems face a core dilemma:

- Centralised EHR databases are high-value breach targets
- Blockchain health records expose PII permanently
- Inter-hospital data sharing requires full trust in intermediaries
- Research data needs de-identification — destroying clinical precision

Arcium MXE enables a third path: compute on encrypted records without ever decrypting them.

---

## Architecture

```
Patient submits health record request
  │
  ├─ Encrypt patient data with MXE public key (x25519-RescueCipher)
  │    Name, DOB, diagnoses, lab values — never lands on-chain unencrypted
  │
  └─► Solana Program (privatemed)
        │  store: encrypted_record + record_commitment
        │
        └─► Arcium MXE (cluster offset: 456)
              │  run compliance and eligibility rules on encrypted data
              │  check treatment eligibility, drug interactions
              │  produce result_hash + proof
              │
              └─► Solana (record_result)
                    │  store: eligible=true/false, expires_at, mxe_proof_hash
                    └─ zero PII stored on-chain

Research / Insurance integration:
  protocol.verify_eligibility(patientPubkey) → true/false
  ↓
  Approved access without raw data exposure
```

---

## On-chain Instructions

| Instruction | Description |
|---|---|
| `register_provider` | Register healthcare provider with MXE routing |
| `submit_record` | Submit MXE-encrypted patient data for computation |
| `record_result` | Write MXE result (eligibility hash + proof) on-chain |
| `verify_eligibility` | External protocols call to gate access |
| `revoke_record` | Revoke on patient request or expiry |

---

## What Is Never On-Chain

- Patient name, date of birth, address
- Diagnoses, medications, lab results
- Insurance policy details
- Any raw clinical data

## What Is On-Chain

- `eligible: bool`
- `expires_at: i64` (Unix timestamp)
- `mxe_proof_hash: [u8; 32]` (Arcium MXE result commitment)
- `record_type: u8` (bitmask: treatment / research / insurance)

---

## Tech Stack

- **Solana** + Anchor Framework 0.32.1
- **Arcium MXE** — Multi-party Execution Environment (cluster 456)
- **ARCIS** circuit DSL — encrypted record eligibility logic
- **x25519-RescueCipher** — client-side patient data encryption
- **TypeScript** client SDK

---

## Setup

```bash
npm install
cp .env.example .env
# Set RPC_URL and WALLET_KEYPAIR_PATH in .env
npx ts-node scripts/submit_record.ts
```

---

## Wallet

`4Y8R73V9QpmL2oUtS4LrwdZk3LrPRCLp7KGg2npPkB1u`

---

## RTG Evidence

See `evidence/` for devnet computation logs and activity reports.
