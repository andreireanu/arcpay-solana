use crate::auth::auth_accept_offer::verify_accept_offer_auth;
use crate::errors::ArcPayError;
use crate::state::{Config, OfferAccepted, SellerVault};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;

#[derive(Accounts)]
pub struct AcceptOffer<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [b"vault", seller.key().as_ref()],
        bump = vault.bump,
        constraint = vault.seller == seller.key() @ ArcPayError::Unauthorized,
    )]
    pub vault: Account<'info, SellerVault>,

    /// CHECK: instructions sysvar, used to verify the prepended ed25519 instruction
    #[account(address = sysvar::instructions::ID)]
    pub instructions_sysvar: UncheckedAccount<'info>,
}

pub fn handler(
    ctx: Context<AcceptOffer>,
    uuid: [u8; 16],
    seller_amount: u64,
    fee_amount: u64,
    expiry: i64,
) -> Result<()> {
    require!(seller_amount > 0, ArcPayError::InvalidAmount);

    let total_amount = seller_amount
        .checked_add(fee_amount)
        .ok_or(ArcPayError::InvalidAmount)?;

    verify_accept_offer_auth(
        &ctx.accounts.instructions_sysvar,
        &ctx.accounts.config.backend_pubkey,
        &ctx.accounts.seller.key(),
        &uuid,
        seller_amount,
        fee_amount,
        expiry,
    )?;

    let vault_info = ctx.accounts.vault.to_account_info();
    let rent_exempt = Rent::get()?.minimum_balance(vault_info.data_len());
    require!(
        vault_info.lamports() >= rent_exempt + total_amount,
        ArcPayError::InsufficientVaultBalance
    );

    // Raw lamport transfer from PDA vault — system_program::transfer cannot be
    // used for PDA accounts that hold data. The vault payout is split between
    // the seller and the protocol fee that accrues on the config account.
    **vault_info.try_borrow_mut_lamports()? -= total_amount;
    **ctx
        .accounts
        .seller
        .to_account_info()
        .try_borrow_mut_lamports()? += seller_amount;
    if fee_amount > 0 {
        **ctx
            .accounts
            .config
            .to_account_info()
            .try_borrow_mut_lamports()? += fee_amount;
    }

    emit!(OfferAccepted {
        uuid,
        seller: ctx.accounts.seller.key(),
        seller_amount,
        fee_amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
