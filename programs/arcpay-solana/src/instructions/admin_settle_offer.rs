use anchor_lang::prelude::*;
use crate::constants::TX_FEE;
use crate::errors::ArcPayError;
use crate::state::{Config, OfferBought, OfferRecord, OfferRefunded};

/// Backend-driven settlement of a single offer record. The direction is a flag:
///   to_seller = true  — accepted offer: amount − fee → seller, fee → config
///   to_seller = false — refund (listing cancel / expiry): amount → buyer
/// Payout targets and amount come from the record only, so even a compromised
/// backend can only route escrow to the seller the buyer chose, or back to the
/// buyer. In both cases `close = buyer` returns the rent to the buyer, minus the
/// TX_FEE the backend recovers for paying the network fee.
#[derive(Accounts)]
#[instruction(uuid: [u8; 16])]
pub struct AdminSettleOffer<'info> {
    #[account(
        mut,
        constraint = backend.key() == config.backend_pubkey @ ArcPayError::Unauthorized,
    )]
    pub backend: Signer<'info>,

    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        close = buyer,
        seeds = [b"offer", uuid.as_ref()],
        bump = offer_record.bump,
    )]
    pub offer_record: Account<'info, OfferRecord>,

    #[account(
        mut,
        constraint = buyer.key() == offer_record.buyer @ ArcPayError::Unauthorized,
    )]
    pub buyer: SystemAccount<'info>,

    #[account(
        mut,
        constraint = seller.key() == offer_record.seller @ ArcPayError::Unauthorized,
    )]
    pub seller: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<AdminSettleOffer>,
    uuid: [u8; 16],
    to_seller: bool,
    fee_amount: u64,
) -> Result<()> {
    let amount = ctx.accounts.offer_record.amount;
    let offer_record_info = ctx.accounts.offer_record.to_account_info();
    let timestamp = Clock::get()?.unix_timestamp;

    // The record holds rent + escrowed amount; everything paid out here comes
    // from the record itself.
    require!(
        offer_record_info.lamports() > amount + TX_FEE,
        ArcPayError::InvalidAmount
    );

    if to_seller {
        require!(fee_amount <= amount, ArcPayError::InvalidAmount);
        let seller_amount = amount - fee_amount;

        **offer_record_info.try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.seller.to_account_info().try_borrow_mut_lamports()? += seller_amount;
        if fee_amount > 0 {
            **ctx.accounts.config.to_account_info().try_borrow_mut_lamports()? += fee_amount;
        }

        emit!(OfferBought {
            uuid,
            buyer: ctx.accounts.buyer.key(),
            seller: ctx.accounts.seller.key(),
            seller_amount,
            fee_amount,
            timestamp,
        });
    } else {
        require!(fee_amount == 0, ArcPayError::InvalidAmount);

        **offer_record_info.try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.buyer.to_account_info().try_borrow_mut_lamports()? += amount;

        emit!(OfferRefunded {
            uuid,
            buyer: ctx.accounts.buyer.key(),
            seller: ctx.accounts.seller.key(),
            amount,
            timestamp,
        });
    }

    // The backend recovers TX_FEE to cover the network fee it pays as fee payer.
    // `close = buyer` then returns the remaining rent to the buyer.
    **offer_record_info.try_borrow_mut_lamports()? -= TX_FEE;
    **ctx.accounts.backend.to_account_info().try_borrow_mut_lamports()? += TX_FEE;

    Ok(())
}
