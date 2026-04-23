use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions::load_instruction_at_checked;
use crate::errors::ArcPayError;

const ED25519_PROGRAM_ID: Pubkey = pubkey!("Ed25519SigVerify111111111111111111111111111");

pub fn verify_backend_auth(
    instructions_sysvar: &AccountInfo,
    backend_pubkey: &Pubkey,
    wallet: &Pubkey,
    uuid: &[u8; 16],
    expiry: i64,
) -> Result<()> {
    // Check the token hasn't expired
    let clock = Clock::get()?;
    require!(clock.unix_timestamp < expiry, ArcPayError::AuthorizationExpired);

    // Scan for the ed25519 instruction — wallet adapters (e.g. Phantom) may prepend
    // compute budget instructions, so it is not guaranteed to be at index 0.
    let mut ed25519_ix = None;
    let mut i: usize = 0;
    loop {
        match load_instruction_at_checked(i, instructions_sysvar) {
            Ok(ix) if ix.program_id == ED25519_PROGRAM_ID => {
                ed25519_ix = Some(ix);
                break;
            }
            Ok(_) => i += 1,
            Err(_) => break,
        }
    }
    let ix = ed25519_ix.ok_or(error!(ArcPayError::MissingEd25519Instruction))?;

    let data = &ix.data;

    // Ed25519 instruction data layout (count and padding are u8, not u16):
    // byte 0        count (u8)
    // byte 1        padding (u8)
    // byte 2-3      signature_offset
    // byte 4-5      signature_instruction_index
    // byte 6-7      public_key_offset
    // byte 8-9      public_key_instruction_index
    // byte 10-11    message_data_offset
    // byte 12-13    message_data_size
    // byte 14-15    message_instruction_index
    // byte 16+      signature(64) | pubkey(32) | message(56)
    require!(data.len() >= 16, ArcPayError::InvalidAuthorizationSignature);

    let count = data[0];
    require!(count >= 1, ArcPayError::InvalidAuthorizationSignature);

    // Assert all data is inline in ix[0] (instruction indices == 0xFFFF).
    // Prevents an attacker from pointing signature/pubkey/message at a different
    // instruction where they might control the bytes at the right offset.
    let sig_ix_idx = u16::from_le_bytes([data[4],  data[5]]);
    let pk_ix_idx  = u16::from_le_bytes([data[8],  data[9]]);
    let msg_ix_idx = u16::from_le_bytes([data[14], data[15]]);
    require!(
        sig_ix_idx == u16::MAX && pk_ix_idx == u16::MAX && msg_ix_idx == u16::MAX,
        ArcPayError::InvalidAuthorizationSignature
    );

    let pk_offset  = u16::from_le_bytes([data[6],  data[7]])  as usize;
    let msg_offset = u16::from_le_bytes([data[10], data[11]]) as usize;
    let msg_size   = u16::from_le_bytes([data[12], data[13]]) as usize;

    require!(data.len() >= pk_offset  + 32,      ArcPayError::InvalidAuthorizationSignature);
    require!(data.len() >= msg_offset + msg_size, ArcPayError::InvalidAuthorizationSignature);
    require!(msg_size == 56,                      ArcPayError::InvalidAuthorizationSignature);

    // Check the pubkey that signed matches the backend keypair stored in Config.
    // Ensures the signature came from our server, not an arbitrary keypair.
    let pk = Pubkey::try_from(&data[pk_offset..pk_offset + 32])
        .map_err(|_| error!(ArcPayError::InvalidAuthorizationSignature))?;
    require_keys_eq!(pk, *backend_pubkey, ArcPayError::InvalidAuthorizationSignature);

    // Extract the signed message and verify each field matches the instruction arguments.
    // The precompile already verified the signature over this message — we just confirm
    // the backend signed the right values, not arbitrary ones the caller supplied.
    let signed_message = &data[msg_offset..msg_offset + 56];

    // [0..32]  wallet — must match the transaction signer
    require!(signed_message[0..32] == wallet.as_ref()[..], ArcPayError::InvalidAuthorizationSignature);

    // [32..48] uuid   — must match the uuid argument passed to register()
    require!(signed_message[32..48] == uuid[..], ArcPayError::InvalidAuthorizationSignature);

    // [48..56] expiry — must match the expiry argument passed to register()
    require!(signed_message[48..56] == expiry.to_le_bytes()[..], ArcPayError::InvalidAuthorizationSignature);

    Ok(())
}
