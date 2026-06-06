use anchor_lang::prelude::*;

pub mod auth;
pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("6KELPQtzV7uchqQC2xEAL7u2BR2Hh5cP57YYUgRpnsed");

#[program]
pub mod arcpay_solana {
    use super::*;

    pub fn initialize_config(ctx: Context<InitializeConfig>, backend_pubkey: Pubkey) -> Result<()> {
        instructions::initialize_config::handler(ctx, backend_pubkey)
    }

    pub fn buy(
        ctx: Context<Buy>,
        seller_amount: u64,
        fee_amount: u64,
        offer_id: [u8; 16],
        expiry: i64,
    ) -> Result<()> {
        instructions::buy::handler(ctx, seller_amount, fee_amount, offer_id, expiry)
    }

    pub fn withdraw_commission(ctx: Context<WithdrawCommission>, amount: u64) -> Result<()> {
        instructions::withdraw_commission::handler(ctx, amount)
    }

    pub fn offer(ctx: Context<Offer>, uuid: [u8; 16], amount: u64, expiry: i64) -> Result<()> {
        instructions::offer::handler(ctx, uuid, amount, expiry)
    }

    pub fn accept_offer(
        ctx: Context<AcceptOffer>,
        uuid: [u8; 16],
        total_amount: u64,
        expiry: i64,
    ) -> Result<()> {
        instructions::accept_offer::handler(ctx, uuid, total_amount, expiry)
    }

    pub fn buyer_cancel_offer(ctx: Context<BuyerCancelOffer>, uuid: [u8; 16]) -> Result<()> {
        instructions::buyer_cancel_offer::handler(ctx, uuid)
    }

    pub fn seller_cancel_offer(ctx: Context<SellerCancelOffer>, offer_id: [u8; 16]) -> Result<()> {
        instructions::seller_cancel_offer::handler(ctx, offer_id)
    }

    pub fn admin_refund_offer(ctx: Context<AdminRefundOffer>, uuid: [u8; 16]) -> Result<()> {
        instructions::admin_refund_offer::handler(ctx, uuid)
    }
}
