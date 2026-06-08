use anchor_lang::prelude::*;
use crate::constants::TX_FEE;
use crate::errors::ArcPayError;
use crate::state::{Config, OfferAdminRefunded, OfferRecord, SellerVault};

#[derive(Accounts)]
#[instruction(uuid: [u8; 16])]
pub struct AdminRefundOffer<'info> {
    #[account(
        mut,
        constraint = backend.key() == config.backend_pubkey @ ArcPayError::Unauthorized,
    )]
    pub backend: Signer<'info>,

    #[account(seeds = [b"config"], bump = config.bump)]
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

    /// None for the accepted offer case (amount already paid to seller, only rent returned).
    /// Some for the listing cancel case (amount + rent returned to buyer).
    #[account(mut)]
    pub vault: Option<Account<'info, SellerVault>>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AdminRefundOffer>, uuid: [u8; 16]) -> Result<()> {
    let amount = ctx.accounts.offer_record.amount;
    let buyer_info = ctx.accounts.buyer.to_account_info();
    let backend_info = ctx.accounts.backend.to_account_info();
    let offer_record_info = ctx.accounts.offer_record.to_account_info();
    let seller = ctx.accounts.offer_record.seller;

    if let Some(vault) = ctx.accounts.vault.as_ref() {
        require!(
            vault.seller == seller,
            ArcPayError::Unauthorized
        );

        let vault_info = vault.to_account_info();
        let rent_exempt = Rent::get()?.minimum_balance(vault_info.data_len());
        require!(
            vault_info.lamports() >= rent_exempt + amount,
            ArcPayError::InsufficientVaultBalance
        );

        // Return the full offer amount from the vault to the buyer.
        **vault_info.try_borrow_mut_lamports()? -= amount;
        **buyer_info.try_borrow_mut_lamports()? += amount;
    }

    // In both cases the backend recovers TX_FEE to cover the network fee it pays
    // as fee payer. Take it from the offer_record rent; `close = buyer` then
    // returns the remaining rent to the buyer.
    require!(
        offer_record_info.lamports() > TX_FEE,
        ArcPayError::InvalidAmount
    );
    **offer_record_info.try_borrow_mut_lamports()? -= TX_FEE;
    **backend_info.try_borrow_mut_lamports()? += TX_FEE;

    emit!(OfferAdminRefunded {
        uuid,
        buyer: ctx.accounts.buyer.key(),
        seller,
        amount: if ctx.accounts.vault.is_some() { amount } else { 0 },
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
