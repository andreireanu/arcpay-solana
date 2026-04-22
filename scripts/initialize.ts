import * as anchor from '@coral-xyz/anchor'

const provider = anchor.AnchorProvider.env()
anchor.setProvider(provider)

const program = anchor.workspace.ArcpaySolana

await program.methods.initializeConfig().rpc()
console.log('config initialized, admin:', provider.wallet.publicKey.toBase58())
