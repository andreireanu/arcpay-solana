# arcpay-solana — Anchor Program Context

## What this program does

On-chain program for Arc Pay, a peer-to-peer crypto payments platform on Solana. Sellers list items with a fixed SOL price. Buyers pay directly on-chain. The program validates all state transitions and emits events for off-chain indexing via Helius webhooks.

---

## Program ID

`DHrS31gfhSxG6RHFgyDnDkAdn8wWkxFtKXpf9hEQ84Kn` (devnet/localnet)

---

## Account Structure

### `Config` — PDA `["config"]`
Program-wide settings. Initialized once after deploy by the admin.

| Field | Type | Description |
|---|---|---|
| `admin` | `Pubkey` | Admin wallet, only one allowed to update config |
| `commission_bps` | `u16` | Commission in basis points (default: 100 = 1%) |
| `bump` | `u8` | PDA bump |

### `UserProfile` — PDA `["user", wallet]`
Created when a user registers. Acts as an on-chain whitelist entry — its existence means the wallet is registered.

| Field | Type | Description |
|---|---|---|
| `wallet` | `Pubkey` | The registered wallet |
| `listing_count` | `u64` | Incremented on each `create_listing`, used as listing PDA nonce |
| `bump` | `u8` | PDA bump |

### `Listing` — PDA `["listing", seller, listing_count_at_creation]`
Created by a registered seller. Represents a for-sale item. Deleted when accepted or cancelled.

| Field | Type | Description |
|---|---|---|
| `seller` | `Pubkey` | Seller wallet |
| `price_lamports` | `u64` | Fixed price in lamports |
| `is_active` | `bool` | False when paused |
| `bump` | `u8` | PDA bump |

---

## Instructions

### Admin
| Instruction | Description |
|---|---|
| `initialize_config` | One-time post-deploy setup. Hardcodes `commission_bps = 100`. Caller becomes admin. |
| `update_config(commission_bps)` | Updates commission rate. Validates `commission_bps <= 10_000`. Admin only. |

### Seller (requires registered `UserProfile`)
| Instruction | Description |
|---|---|
| `register` | Creates a `UserProfile` PDA for the signing wallet. Anchor's `init` prevents double registration. |
| `create_listing(price_lamports)` | Creates a `Listing` PDA. Increments `seller_profile.listing_count`. |
| `pause_listing` | Sets `listing.is_active = false`. Buyer cannot accept a paused listing. |
| `resume_listing` | Sets `listing.is_active = true`. |
| `cancel_listing` | Closes the listing account. Rent returned to seller. |

### Buyer
| Instruction | Description |
|---|---|
| `accept_listing` | Validates listing is active. Transfers `price_lamports` from buyer to seller. Closes the listing PDA (rent also goes to seller). Emits `ListingAccepted`. |

---

## Events

All events are emitted via Anchor's `emit!()` macro and indexed by Helius webhooks to write orders into Supabase.

| Event | Emitted by | Key Fields |
|---|---|---|
| `ListingCreated` | `create_listing` | listing, seller, price_lamports, timestamp |
| `ListingPaused` | `pause_listing` | listing, seller, timestamp |
| `ListingResumed` | `resume_listing` | listing, seller, timestamp |
| `ListingCancelled` | `cancel_listing` | listing, seller, timestamp |
| `ListingAccepted` | `accept_listing` | listing, buyer, seller, amount_lamports, timestamp |

---

## Key Design Decisions

- **Listing PDA as identifier** — the listing's PDA address is the unique identifier used in QR codes and indexed in Supabase. No separate UUID needed.
- **Price enforced on-chain** — price is stored in the `Listing` account; no backend co-signing required to prevent tampering.
- **Listing deleted on accept** — `accept_listing` closes the listing via `close = seller`. It acts as a temporary witness; what matters is the SOL transfer and the event.
- **No escrow** — SOL goes directly from buyer to seller. Dispute resolution is out of scope for this version.
- **Registration as whitelist** — a `UserProfile` PDA's existence is the whitelist check. No separate list account needed.

---

## File Structure

```
programs/arcpay-solana/src/
├── lib.rs                          # Program entrypoints
├── state.rs                        # Account structs + events
├── errors.rs                       # Custom error codes
└── instructions/
    ├── mod.rs
    ├── initialize_config.rs
    ├── update_config.rs
    ├── register.rs
    ├── create_listing.rs
    ├── pause_listing.rs
    ├── resume_listing.rs
    ├── cancel_listing.rs
    └── accept_listing.rs
```

---

## What's Not Implemented Yet

- Dispute resolution
- Commission collection (currently tracked in Config but not deducted on `accept_listing`)
- Admin transferability
