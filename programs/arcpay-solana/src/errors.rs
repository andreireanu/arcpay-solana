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
}
