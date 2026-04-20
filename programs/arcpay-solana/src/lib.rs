use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("DHrS31gfhSxG6RHFgyDnDkAdn8wWkxFtKXpf9hEQ84Kn");

#[program]
pub mod arcpay_solana {
    use super::*;

    pub fn register(ctx: Context<Register>) -> Result<()> {
        instructions::register::handler(ctx)
    }

    pub fn make_offer(ctx: Context<MakeOffer>) -> Result<()> {
        instructions::make_offer::handler(ctx)
    }

    pub fn take_offer(ctx: Context<TakeOffer>) -> Result<()> {
        instructions::take_offer::handler(ctx)
    }
}
