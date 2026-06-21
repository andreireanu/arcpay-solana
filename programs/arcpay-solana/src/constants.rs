//! Program-wide constants.

use anchor_lang::prelude::*;

/// Solana's native Ed25519 signature-verification program. The `auth` layer
/// scans the transaction's instructions for one targeting this program and
/// reads the backend signature/pubkey/message out of its data.
pub const ED25519_PROGRAM_ID: Pubkey = pubkey!("Ed25519SigVerify111111111111111111111111111");

/// Lamports the backend reclaims from a settled escrow to cover the network fee
/// it pays as the settlement transaction's fee payer.
pub const TX_FEE: u64 = 5_000;
