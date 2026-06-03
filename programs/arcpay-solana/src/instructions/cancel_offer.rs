use anchor_lang::prelude::*;
use crate::errors::ArcPayError;
use crate::state::{OfferCanceled, OfferRecord, SellerVault};

#[derive(Accounts)]
#[instruction(uuid: [u8; 16])]
pub struct CancelOffer<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: Used only for deriving the seller's vault address. Verified by offer_record constraint.
    pub seller: UncheckedAccount<'info>,

    #[account(
        mut,
        close = buyer,
        seeds = [b"offer", uuid.as_ref()],
        bump = offer_record.bump,
        constraint = offer_record.buyer == buyer.key() @ ArcPayError::Unauthorized,
        constraint = offer_record.seller == seller.key() @ ArcPayError::Unauthorized,
    )]
    pub offer_record: Account<'info, OfferRecord>,

    #[account(
        mut,
        seeds = [b"vault", seller.key().as_ref()],
        bump = vault.bump,
        constraint = vault.seller == seller.key() @ ArcPayError::Unauthorized,
    )]
    pub vault: Account<'info, SellerVault>,
}

pub fn handler(ctx: Context<CancelOffer>, uuid: [u8; 16]) -> Result<()> {
    let amount = ctx.accounts.offer_record.amount;

    let vault_info = ctx.accounts.vault.to_account_info();
    let rent_exempt = Rent::get()?.minimum_balance(vault_info.data_len());
    
    require!(
        vault_info.lamports() >= rent_exempt + amount,
        ArcPayError::InsufficientVaultBalance
    );

    // Raw lamport transfer from PDA vault back to buyer
    **vault_info.try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.buyer.to_account_info().try_borrow_mut_lamports()? += amount;

    emit!(OfferCanceled {
        uuid,
        buyer: ctx.accounts.buyer.key(),
        seller: ctx.accounts.seller.key(),
        amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
