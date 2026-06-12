import * as anchor from '@coral-xyz/anchor'
import { Keypair, PublicKey } from '@solana/web3.js'
import { readFileSync } from 'fs'
import { homedir } from 'os'

const UPGRADEABLE_LOADER = new PublicKey('BPFLoaderUpgradeab1e11111111111111111111111')

const provider = anchor.AnchorProvider.env()
anchor.setProvider(provider)

const program = anchor.workspace.ArcpaySolana

const backendKeypairBytes = JSON.parse(
  readFileSync(`${homedir()}/.config/solana/arcpay-backend.json`, 'utf8')
)
const backendKeypair = Keypair.fromSecretKey(new Uint8Array(backendKeypairBytes));

(async () => {
  // initialize_config is restricted to the program's upgrade authority; the
  // ProgramData account (where that authority lives) is a PDA of the loader.
  const [programData] = PublicKey.findProgramAddressSync(
    [program.programId.toBuffer()],
    UPGRADEABLE_LOADER,
  )

  await program.methods
    .initializeConfig(backendKeypair.publicKey)
    .accounts({ admin: provider.wallet.publicKey, programData })
    .rpc()

  console.log('config initialized')
  console.log('admin:          ', provider.wallet.publicKey.toBase58())
  console.log('backend pubkey: ', backendKeypair.publicKey.toBase58())
})()
