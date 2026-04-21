use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("DHrS31gfhSxG6RHFgyDnDkAdn8wWkxFtKXpf9hEQ84Kn");

#[program]
pub mod arcpay_solana {
    use super::*;

    pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
        instructions::initialize_config::handler(ctx)
    }

    pub fn update_config(ctx: Context<UpdateConfig>, commission_bps: u16) -> Result<()> {
        instructions::update_config::handler(ctx, commission_bps)
    }

    pub fn register(ctx: Context<Register>) -> Result<()> {
        instructions::register::handler(ctx)
    }

    pub fn create_listing(ctx: Context<CreateListing>, price_lamports: u64) -> Result<()> {
        instructions::create_listing::handler(ctx, price_lamports)
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
}
