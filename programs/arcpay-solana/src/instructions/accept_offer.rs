use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;
use crate::auth_accept_offer::verify_accept_offer_auth;
use crate::errors::ArcPayError;
use crate::state::{Config, SellerVault, OfferAccepted};

#[derive(Accounts)]
pub struct AcceptOffer<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(seeds = [b"config"], bump = config.bump)]
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

pub fn handler(ctx: Context<AcceptOffer>, uuid: [u8; 16], total_amount: u64, expiry: i64) -> Result<()> {
    require!(total_amount > 0, ArcPayError::InvalidAmount);

    verify_accept_offer_auth(
        &ctx.accounts.instructions_sysvar,
        &ctx.accounts.config.backend_pubkey,
        &ctx.accounts.seller.key(),
        &uuid,
        total_amount,
        expiry,
    )?;

    let vault_info = ctx.accounts.vault.to_account_info();
    let rent_exempt = Rent::get()?.minimum_balance(vault_info.data_len());
    require!(
        vault_info.lamports() >= rent_exempt + total_amount,
        ArcPayError::InsufficientVaultBalance
    );

    // Raw lamport transfer from PDA vault to seller — system_program::transfer
    // cannot be used for PDA accounts that hold data.
    **vault_info.try_borrow_mut_lamports()? -= total_amount;
    **ctx.accounts.seller.to_account_info().try_borrow_mut_lamports()? += total_amount;

    emit!(OfferAccepted {
        uuid,
        seller: ctx.accounts.seller.key(),
        total_amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
