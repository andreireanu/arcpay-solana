use anchor_lang::prelude::*;

#[error_code]
pub enum ArcPayError {
    #[msg("Wallet already registered")]
    AlreadyRegistered,
    #[msg("Wallet not registered")]
    NotRegistered,
    #[msg("Unauthorized: admin only")]
    Unauthorized,
    #[msg("Commission must be between 0 and 10000 basis points")]
    InvalidCommission,
    #[msg("Listing is not active")]
    ListingNotActive,
    #[msg("Authorization signature has expired")]
    AuthorizationExpired,
    #[msg("Ed25519 instruction missing or not at index 0")]
    MissingEd25519Instruction,
    #[msg("Invalid backend authorization signature")]
    InvalidAuthorizationSignature,
    #[msg("Withdrawal amount exceeds available commission balance")]
    InsufficientCommissionBalance,
}
