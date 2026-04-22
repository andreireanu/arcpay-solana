import * as anchor from '@coral-xyz/anchor'

const provider = anchor.AnchorProvider.env()
anchor.setProvider(provider)

const program = anchor.workspace.ArcpaySolana
const admin = provider.wallet.publicKey
const wallet = new anchor.web3.PublicKey(process.argv[2] ?? admin.toBase58())

await program.methods.removeRegistration().accounts({ wallet }).rpc()
console.log('deregistered:', wallet.toBase58())
