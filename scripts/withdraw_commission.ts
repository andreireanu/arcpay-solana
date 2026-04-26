import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.ArcpaySolana;

const [configPda] = anchor.web3.PublicKey.findProgramAddressSync(
  [Buffer.from("config")],
  program.programId,
);

(async () => {
  const configAccount = await provider.connection.getAccountInfo(configPda);
  if (!configAccount) {
    console.error("Config PDA not found. Has initialize_config been run?");
    process.exit(1);
  }

  const rentExempt = await provider.connection.getMinimumBalanceForRentExemption(
    configAccount.data.length,
  );
  const available = configAccount.lamports - rentExempt;

  if (available <= 0) {
    console.log("No commission available to withdraw.");
    process.exit(0);
  }

  const amountArg = process.argv[2];
  const amount = amountArg ? new BN(amountArg) : new BN(available);

  console.log(`Config PDA:  ${configPda.toBase58()}`);
  console.log(`Available:   ${available} lamports (${available / anchor.web3.LAMPORTS_PER_SOL} SOL)`);
  console.log(`Withdrawing: ${amount.toString()} lamports (${amount.toNumber() / anchor.web3.LAMPORTS_PER_SOL} SOL)`);

  const tx = await program.methods
    .withdrawCommission(amount)
    .accounts({ admin: provider.wallet.publicKey })
    .rpc();

  console.log("Withdrawn. Tx:", tx);
})();
