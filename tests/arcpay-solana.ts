import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import {
  ComputeBudgetProgram,
  Ed25519Program,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SYSVAR_INSTRUCTIONS_PUBKEY,
} from "@solana/web3.js";
import assert from "assert";
import crypto from "crypto";
import { ArcpaySolana } from "../target/types/arcpay_solana";

const TX_FEE = 5_000;
const RECORD_SPACE = 8 + 32 + 32 + 8 + 1; // discriminator + OfferRecord

describe("arcpay-solana", () => {
  // Read everything at `confirmed`; the env default (`processed`) makes
  // before/after balance assertions race the cluster.
  const env = anchor.AnchorProvider.env();
  const provider = new anchor.AnchorProvider(env.connection, env.wallet, {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
  });
  anchor.setProvider(provider);
  const connection = provider.connection;

  const program = anchor.workspace.arcpaySolana as Program<ArcpaySolana>;
  const eventParser = new anchor.EventParser(
    program.programId,
    new anchor.BorshCoder(program.idl),
  );

  const backend = Keypair.generate();
  const buyer = Keypair.generate();
  const seller = Keypair.generate();

  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    program.programId,
  );

  let recordRent = 0;

  // --- helpers -------------------------------------------------------------

  const newUuid = () => crypto.randomBytes(16);

  const offerPda = (uuid: Buffer) =>
    PublicKey.findProgramAddressSync(
      [Buffer.from("offer"), uuid],
      program.programId,
    )[0];

  const nowPlus = (secs: number) => new BN(Math.floor(Date.now() / 1000) + secs);

  // Message layout (96 bytes): buyer | seller | uuid | amount LE | expiry LE
  function offerMsg(
    buyerKey: PublicKey,
    sellerKey: PublicKey,
    uuid: Buffer,
    amount: BN,
    expiry: BN,
  ): Buffer {
    return Buffer.concat([
      buyerKey.toBuffer(),
      sellerKey.toBuffer(),
      uuid,
      amount.toArrayLike(Buffer, "le", 8),
      expiry.toArrayLike(Buffer, "le", 8),
    ]);
  }

  // Message layout (56 bytes): seller | uuid | expiry LE
  function acceptMsg(sellerKey: PublicKey, uuid: Buffer, expiry: BN): Buffer {
    return Buffer.concat([
      sellerKey.toBuffer(),
      uuid,
      expiry.toArrayLike(Buffer, "le", 8),
    ]);
  }

  const backendSigIx = (message: Buffer) =>
    Ed25519Program.createInstructionWithPrivateKey({
      privateKey: backend.secretKey,
      message,
    });

  async function airdrop(to: PublicKey, sol: number) {
    const sig = await connection.requestAirdrop(to, sol * LAMPORTS_PER_SOL);
    const latest = await connection.getLatestBlockhash();
    await connection.confirmTransaction({ signature: sig, ...latest }, "confirmed");
  }

  async function balance(key: PublicKey): Promise<number> {
    return connection.getBalance(key, "confirmed");
  }

  async function fetchEvents(signature: string) {
    const latest = await connection.getLatestBlockhash();
    await connection.confirmTransaction({ signature, ...latest }, "confirmed");
    const tx = await connection.getTransaction(signature, {
      commitment: "confirmed",
      maxSupportedTransactionVersion: 0,
    });
    return [...eventParser.parseLogs(tx!.meta!.logMessages!)];
  }

  async function expectFail(p: Promise<unknown>, code: string) {
    try {
      await p;
      assert.fail(`expected failure with ${code}`);
    } catch (e: any) {
      const got = e?.error?.errorCode?.code ?? "";
      assert(
        got === code || e.toString().includes(code),
        `expected ${code}, got: ${e}`,
      );
    }
  }

  /** Create a backend-authorized offer; returns its uuid. */
  async function makeOffer(amount: BN): Promise<Buffer> {
    const uuid = newUuid();
    const expiry = nowPlus(300);
    const msg = offerMsg(buyer.publicKey, seller.publicKey, uuid, amount, expiry);

    await program.methods
      .offer([...uuid], amount, expiry)
      .accountsPartial({
        buyer: buyer.publicKey,
        seller: seller.publicKey,
        config: configPda,
        offerRecord: offerPda(uuid),
        instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .preInstructions([backendSigIx(msg)])
      .signers([buyer])
      .rpc();

    return uuid;
  }

  function settleIx(uuid: Buffer, toSeller: boolean, fee: BN) {
    return program.methods
      .adminSettleOffer([...uuid], toSeller, fee)
      .accountsPartial({
        backend: backend.publicKey,
        config: configPda,
        offerRecord: offerPda(uuid),
        buyer: buyer.publicKey,
        seller: seller.publicKey,
      })
      .signers([backend]);
  }

  // --- setup ---------------------------------------------------------------

  before(async () => {
    await airdrop(buyer.publicKey, 100);
    // the backend receives TX_FEE carve-outs; it must already be rent-exempt
    await airdrop(backend.publicKey, 1);
    recordRent = await connection.getMinimumBalanceForRentExemption(RECORD_SPACE);

    // init is restricted to the upgrade authority — in `anchor test` that is
    // the provider wallet, which deployed the program to the local validator
    const [programData] = PublicKey.findProgramAddressSync(
      [program.programId.toBuffer()],
      new PublicKey("BPFLoaderUpgradeab1e11111111111111111111111"),
    );

    // front-running guard: a non-authority caller must be rejected. Has to run
    // before the real init — afterwards any attempt fails on "already in use"
    // and wouldn't exercise the authority constraint.
    const impostor = Keypair.generate();
    await airdrop(impostor.publicKey, 1);
    try {
      await program.methods
        .initializeConfig(impostor.publicKey)
        .accountsPartial({ admin: impostor.publicKey, programData })
        .signers([impostor])
        .rpc();
      assert.fail("impostor was able to initialize config");
    } catch (e: any) {
      const got = e?.error?.errorCode?.code ?? String(e);
      assert(String(got).includes("Unauthorized"), `expected Unauthorized, got: ${e}`);
    }

    await program.methods
      .initializeConfig(backend.publicKey)
      .accountsPartial({ admin: provider.wallet.publicKey, programData })
      .rpc();
  });

  it("initializes config", async () => {
    const config = await program.account.config.fetch(configPda);
    assert(config.admin.equals(provider.wallet.publicKey));
    assert(config.backendPubkey.equals(backend.publicKey));
  });

  // --- offer ---------------------------------------------------------------

  describe("offer", () => {
    it("escrows the amount in the offer record", async () => {
      const amount = new BN(2 * LAMPORTS_PER_SOL);
      const uuid = await makeOffer(amount);

      const record = await program.account.offerRecord.fetch(offerPda(uuid));
      assert(record.buyer.equals(buyer.publicKey));
      assert(record.seller.equals(seller.publicKey));
      assert.equal(record.amount.toString(), amount.toString());

      const lamports = await balance(offerPda(uuid));
      assert.equal(lamports, recordRent + amount.toNumber());
    });

    it("rejects a seller account that differs from the signed message", async () => {
      const uuid = newUuid();
      const amount = new BN(LAMPORTS_PER_SOL);
      const expiry = nowPlus(300);
      const msg = offerMsg(buyer.publicKey, seller.publicKey, uuid, amount, expiry);
      const impostor = Keypair.generate();

      await expectFail(
        program.methods
          .offer([...uuid], amount, expiry)
          .accountsPartial({
            buyer: buyer.publicKey,
            seller: impostor.publicKey, // signed for `seller`, passing someone else
            config: configPda,
            offerRecord: offerPda(uuid),
            instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
          })
          .preInstructions([backendSigIx(msg)])
          .signers([buyer])
          .rpc(),
        "InvalidAuthorizationSignature",
      );
    });

    it("rejects a tampered amount", async () => {
      const uuid = newUuid();
      const expiry = nowPlus(300);
      const msg = offerMsg(
        buyer.publicKey,
        seller.publicKey,
        uuid,
        new BN(LAMPORTS_PER_SOL),
        expiry,
      );

      await expectFail(
        program.methods
          .offer([...uuid], new BN(2 * LAMPORTS_PER_SOL), expiry)
          .accountsPartial({
            buyer: buyer.publicKey,
            seller: seller.publicKey,
            config: configPda,
            offerRecord: offerPda(uuid),
            instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
          })
          .preInstructions([backendSigIx(msg)])
          .signers([buyer])
          .rpc(),
        "InvalidAuthorizationSignature",
      );
    });

    it("rejects an expired authorization", async () => {
      const uuid = newUuid();
      const amount = new BN(LAMPORTS_PER_SOL);
      const expiry = nowPlus(-10);
      const msg = offerMsg(buyer.publicKey, seller.publicKey, uuid, amount, expiry);

      await expectFail(
        program.methods
          .offer([...uuid], amount, expiry)
          .accountsPartial({
            buyer: buyer.publicKey,
            seller: seller.publicKey,
            config: configPda,
            offerRecord: offerPda(uuid),
            instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
          })
          .preInstructions([backendSigIx(msg)])
          .signers([buyer])
          .rpc(),
        "AuthorizationExpired",
      );
    });

    it("rejects without the ed25519 instruction", async () => {
      const uuid = newUuid();
      const amount = new BN(LAMPORTS_PER_SOL);
      const expiry = nowPlus(300);

      await expectFail(
        program.methods
          .offer([...uuid], amount, expiry)
          .accountsPartial({
            buyer: buyer.publicKey,
            seller: seller.publicKey,
            config: configPda,
            offerRecord: offerPda(uuid),
            instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
          })
          .signers([buyer])
          .rpc(),
        "MissingEd25519Instruction",
      );
    });
  });

  // --- accept_offer ----------------------------------------------------------

  describe("accept_offer", () => {
    it("emits the consent event and moves no funds", async () => {
      const uuid = newUuid();
      const expiry = nowPlus(300);
      const msg = acceptMsg(seller.publicKey, uuid, expiry);

      const sellerBefore = await balance(seller.publicKey);
      const configBefore = await balance(configPda);

      const sig = await program.methods
        .acceptOffer([...uuid], expiry)
        .accountsPartial({
          seller: seller.publicKey,
          config: configPda,
          instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
        })
        .preInstructions([backendSigIx(msg)])
        .signers([seller])
        .rpc();

      const events = await fetchEvents(sig);
      const ev = events.find((e) => e.name === "offerAccepted");
      assert(ev, "OfferAccepted not emitted");
      assert(Buffer.from(ev!.data.uuid).equals(uuid));
      assert(ev!.data.seller.equals(seller.publicKey));

      assert.equal(await balance(seller.publicKey), sellerBefore);
      assert.equal(await balance(configPda), configBefore);
    });

    it("tolerates replay: re-emits without effect (and skips prepended instructions)", async () => {
      const uuid = newUuid();
      const expiry = nowPlus(300);
      const msg = acceptMsg(seller.publicKey, uuid, expiry);
      const sellerBefore = await balance(seller.publicKey);

      const accept = () =>
        program.methods
          .acceptOffer([...uuid], expiry)
          .accountsPartial({
            seller: seller.publicKey,
            config: configPda,
            instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
          })
          .preInstructions([
            // compute-budget prefix exercises the sysvar scan
            ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }),
            backendSigIx(msg),
          ])
          .signers([seller]);

      const first = await accept().rpc();
      await fetchEvents(first);
      // a replay is a *new* transaction reusing the same signed message
      const second = await accept()
        .postInstructions([
          ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 1 }),
        ])
        .rpc();

      const events = await fetchEvents(second);
      assert(events.find((e) => e.name === "offerAccepted"));
      assert.equal(await balance(seller.publicKey), sellerBefore);
    });

    it("rejects an expired accept authorization", async () => {
      const uuid = newUuid();
      const expiry = nowPlus(-10);
      const msg = acceptMsg(seller.publicKey, uuid, expiry);

      await expectFail(
        program.methods
          .acceptOffer([...uuid], expiry)
          .accountsPartial({
            seller: seller.publicKey,
            config: configPda,
            instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
          })
          .preInstructions([backendSigIx(msg)])
          .signers([seller])
          .rpc(),
        "AuthorizationExpired",
      );
    });

    it("rejects an accept signed for another seller", async () => {
      const uuid = newUuid();
      const expiry = nowPlus(300);
      const msg = acceptMsg(seller.publicKey, uuid, expiry);
      const impostor = Keypair.generate();

      await expectFail(
        program.methods
          .acceptOffer([...uuid], expiry)
          .accountsPartial({
            seller: impostor.publicKey,
            config: configPda,
            instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
          })
          .preInstructions([backendSigIx(msg)])
          .signers([impostor])
          .rpc(),
        "InvalidAuthorizationSignature",
      );
    });
  });

  // --- admin_settle_offer ----------------------------------------------------

  describe("admin_settle_offer (to_seller = true)", () => {
    it("pays the seller minus fee, fee to config, rent to buyer", async () => {
      const amount = new BN(LAMPORTS_PER_SOL);
      const fee = new BN(LAMPORTS_PER_SOL / 20); // 5%
      const uuid = await makeOffer(amount);

      const sellerBefore = await balance(seller.publicKey);
      const configBefore = await balance(configPda);
      const buyerBefore = await balance(buyer.publicKey);
      const backendBefore = await balance(backend.publicKey);

      const sig = await settleIx(uuid, true, fee).rpc();
      const events = await fetchEvents(sig);

      const ev = events.find((e) => e.name === "offerBought");
      assert(ev, "OfferBought not emitted");
      assert(Buffer.from(ev!.data.uuid).equals(uuid));
      assert.equal(
        ev!.data.sellerAmount.toString(),
        amount.sub(fee).toString(),
      );
      assert.equal(ev!.data.feeAmount.toString(), fee.toString());

      assert.equal(
        await balance(seller.publicKey),
        sellerBefore + amount.toNumber() - fee.toNumber(),
      );
      assert.equal(await balance(configPda), configBefore + fee.toNumber());
      assert.equal(
        await balance(buyer.publicKey),
        buyerBefore + recordRent - TX_FEE,
      );
      assert.equal(await balance(backend.publicKey), backendBefore + TX_FEE);
      assert.equal(await connection.getAccountInfo(offerPda(uuid), "confirmed"), null);
    });

    it("fails when fee exceeds the escrowed amount", async () => {
      const amount = new BN(LAMPORTS_PER_SOL);
      const uuid = await makeOffer(amount);
      await expectFail(
        settleIx(uuid, true, amount.addn(1)).rpc(),
        "InvalidAmount",
      );
      // leave the record clean for other tests
      await settleIx(uuid, true, new BN(0)).rpc();
    });

    it("fails for a non-backend signer", async () => {
      const amount = new BN(LAMPORTS_PER_SOL);
      const uuid = await makeOffer(amount);
      const impostor = Keypair.generate();

      await expectFail(
        program.methods
          .adminSettleOffer([...uuid], true, new BN(0))
          .accountsPartial({
            backend: impostor.publicKey,
            config: configPda,
            offerRecord: offerPda(uuid),
            buyer: buyer.publicKey,
            seller: seller.publicKey,
          })
          .signers([impostor])
          .rpc(),
        "Unauthorized",
      );
      await settleIx(uuid, true, new BN(0)).rpc();
    });

    it("fails on a mismatched seller account", async () => {
      const amount = new BN(LAMPORTS_PER_SOL);
      const uuid = await makeOffer(amount);
      const impostor = Keypair.generate();

      await expectFail(
        program.methods
          .adminSettleOffer([...uuid], true, new BN(0))
          .accountsPartial({
            backend: backend.publicKey,
            config: configPda,
            offerRecord: offerPda(uuid),
            buyer: buyer.publicKey,
            seller: impostor.publicKey,
          })
          .signers([backend])
          .rpc(),
        "Unauthorized",
      );
      await settleIx(uuid, true, new BN(0)).rpc();
    });

    it("fails on the second settle: the record pays out exactly once", async () => {
      const amount = new BN(LAMPORTS_PER_SOL);
      const uuid = await makeOffer(amount);

      await settleIx(uuid, true, new BN(0)).rpc();
      await expectFail(
        settleIx(uuid, true, new BN(0)).rpc(),
        "AccountNotInitialized",
      );
    });
  });

  describe("admin_settle_offer (to_seller = false)", () => {
    it("refunds amount + rent to the buyer", async () => {
      const amount = new BN(LAMPORTS_PER_SOL);
      const uuid = await makeOffer(amount);

      const buyerBefore = await balance(buyer.publicKey);
      const sellerBefore = await balance(seller.publicKey);

      const sig = await settleIx(uuid, false, new BN(0)).rpc();
      const events = await fetchEvents(sig);

      const ev = events.find((e) => e.name === "offerRefunded");
      assert(ev, "OfferRefunded not emitted");
      assert.equal(ev!.data.amount.toString(), amount.toString());

      assert.equal(
        await balance(buyer.publicKey),
        buyerBefore + amount.toNumber() + recordRent - TX_FEE,
      );
      assert.equal(await balance(seller.publicKey), sellerBefore);
      assert.equal(await connection.getAccountInfo(offerPda(uuid)), null);
    });

    it("fails when a fee is passed on the refund path", async () => {
      const amount = new BN(LAMPORTS_PER_SOL);
      const uuid = await makeOffer(amount);
      await expectFail(settleIx(uuid, false, new BN(1)).rpc(), "InvalidAmount");
      await settleIx(uuid, false, new BN(0)).rpc();
    });
  });

  // --- buyer_cancel_offer ------------------------------------------------------

  describe("buyer_cancel_offer", () => {
    it("buyer reclaims escrow + rent", async () => {
      const amount = new BN(LAMPORTS_PER_SOL);
      const uuid = await makeOffer(amount);
      const buyerBefore = await balance(buyer.publicKey);

      const sig = await program.methods
        .buyerCancelOffer([...uuid])
        .accountsPartial({
          buyer: buyer.publicKey,
          offerRecord: offerPda(uuid),
        })
        .signers([buyer])
        .rpc();

      const events = await fetchEvents(sig);
      const ev = events.find((e) => e.name === "buyerOfferCanceled");
      assert(ev, "BuyerOfferCanceled not emitted");
      assert(ev!.data.seller.equals(seller.publicKey));
      assert.equal(ev!.data.amount.toString(), amount.toString());

      assert.equal(
        await balance(buyer.publicKey),
        buyerBefore + amount.toNumber() + recordRent,
      );
      assert.equal(await connection.getAccountInfo(offerPda(uuid)), null);
    });

    it("fails for a non-buyer", async () => {
      const amount = new BN(LAMPORTS_PER_SOL);
      const uuid = await makeOffer(amount);
      const impostor = Keypair.generate();
      await airdrop(impostor.publicKey, 1);

      await expectFail(
        program.methods
          .buyerCancelOffer([...uuid])
          .accountsPartial({
            buyer: impostor.publicKey,
            offerRecord: offerPda(uuid),
          })
          .signers([impostor])
          .rpc(),
        "Unauthorized",
      );
      await settleIx(uuid, false, new BN(0)).rpc();
    });

    it("settle after cancel fails: the race loser gets a clean failure", async () => {
      const amount = new BN(LAMPORTS_PER_SOL);
      const uuid = await makeOffer(amount);

      await program.methods
        .buyerCancelOffer([...uuid])
        .accountsPartial({
          buyer: buyer.publicKey,
          offerRecord: offerPda(uuid),
        })
        .signers([buyer])
        .rpc();

      await expectFail(
        settleIx(uuid, true, new BN(0)).rpc(),
        "AccountNotInitialized",
      );
    });

    it("cancel after settle fails", async () => {
      const amount = new BN(LAMPORTS_PER_SOL);
      const uuid = await makeOffer(amount);

      await settleIx(uuid, true, new BN(0)).rpc();

      await expectFail(
        program.methods
          .buyerCancelOffer([...uuid])
          .accountsPartial({
            buyer: buyer.publicKey,
            offerRecord: offerPda(uuid),
          })
          .signers([buyer])
          .rpc(),
        "AccountNotInitialized",
      );
    });
  });
});
