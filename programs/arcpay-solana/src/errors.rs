use anchor_lang::prelude::*;

#[error_code]
pub enum ArcPayError {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Authorization signature has expired")]
    AuthorizationExpired,
    #[msg("Ed25519 signature verification instruction missing")]
    MissingEd25519Instruction,
    #[msg("Invalid backend authorization signature")]
    InvalidAuthorizationSignature,
    #[msg("Withdrawal amount exceeds available commission balance")]
    InsufficientCommissionBalance,
    #[msg("Invalid amount")]
    InvalidAmount,
}
