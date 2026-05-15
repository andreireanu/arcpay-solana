import * as anchor from '@coral-xyz/anchor'
import { Keypair } from '@solana/web3.js'
import { readFileSync } from 'fs'
import { homedir } from 'os'

const provider = anchor.AnchorProvider.env()
anchor.setProvider(provider)

const program = anchor.workspace.ArcpaySolana

const backendKeypairBytes = JSON.parse(
  readFileSync(`${homedir()}/.config/solana/arcpay-backend.json`, 'utf8')
)
const backendKeypair = Keypair.fromSecretKey(new Uint8Array(backendKeypairBytes));

(async () => {
  await program.methods
    .initializeConfig(backendKeypair.publicKey)
    .accounts({ admin: provider.wallet.publicKey })
    .rpc()

  console.log('config initialized')
  console.log('admin:          ', provider.wallet.publicKey.toBase58())
  console.log('backend pubkey: ', backendKeypair.publicKey.toBase58())
})()
