import * as anchor from '@coral-xyz/anchor'
import { PublicKey } from '@solana/web3.js'

const provider = anchor.AnchorProvider.env()
anchor.setProvider(provider)

const program = anchor.workspace.ArcpaySolana
const admin = provider.wallet.publicKey
const wallet = new PublicKey(process.argv[2] ?? admin.toBase58());

(async () => {
  await program.methods.removeRegistration().accounts({ wallet }).rpc()
  console.log('deregistered:', wallet.toBase58())
})()
