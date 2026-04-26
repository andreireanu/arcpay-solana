use anchor_lang::prelude::*;

pub mod auth;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("4TmDTsMfaftRVdDRuMvpBkJgoTUgxrqC9vE2vNGKqLBp");

#[program]
pub mod arcpay_solana {
    use super::*;

    pub fn initialize_config(ctx: Context<InitializeConfig>, backend_pubkey: Pubkey) -> Result<()> {
        instructions::initialize_config::handler(ctx, backend_pubkey)
    }

    pub fn update_config(ctx: Context<UpdateConfig>, commission_bps: u16) -> Result<()> {
        instructions::update_config::handler(ctx, commission_bps)
    }

    pub fn register(ctx: Context<Register>, expiry: i64, uuid: [u8; 16]) -> Result<()> {
        instructions::register::handler(ctx, expiry, uuid)
    }

    pub fn create_listing(ctx: Context<CreateListing>, price_lamports: u64, uuid: [u8; 16]) -> Result<()> {
        instructions::create_listing::handler(ctx, price_lamports, uuid)
    }

    pub fn pause_listing(ctx: Context<PauseListing>) -> Result<()> {
        instructions::pause_listing::handler(ctx)
    }

    pub fn resume_listing(ctx: Context<ResumeListing>) -> Result<()> {
        instructions::resume_listing::handler(ctx)
    }

    pub fn cancel_listing(ctx: Context<CancelListing>) -> Result<()> {
        instructions::cancel_listing::handler(ctx)
    }

    pub fn accept_listing(ctx: Context<AcceptListing>) -> Result<()> {
        instructions::accept_listing::handler(ctx)
    }

    pub fn remove_registration(ctx: Context<RemoveRegistration>) -> Result<()> {
        instructions::remove_registration::handler(ctx)
    }

    pub fn withdraw_commission(ctx: Context<WithdrawCommission>, amount: u64) -> Result<()> {
        instructions::withdraw_commission::handler(ctx, amount)
    }
}
