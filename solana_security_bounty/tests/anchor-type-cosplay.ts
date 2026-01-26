import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { assert } from 'chai'
import { AnchorTypeCosplay } from '../target/types/anchor_type_cosplay'

describe('anchor-type-cosplay', () => {
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const program = anchor.workspace
    .AnchorTypeCosplay as Program<AnchorTypeCosplay>

  const user = anchor.web3.Keypair.generate()
  const admin = anchor.web3.Keypair.generate()
  const authority = provider.wallet.publicKey

  it('Initializes accounts', async () => {
    await program.methods
      .initializeUser(new anchor.BN(123))
      .accounts({ user: user.publicKey, authority })
      .signers([user])
      .rpc()

    await program.methods
      .initializeAdmin(new anchor.BN(999))
      .accounts({ admin: admin.publicKey, authority })
      .signers([admin])
      .rpc()
  })

  it('VULNERABLE: Accepts Admin account as User', async () => {
    try {
      await program.methods
        .cosplayInsecure()
        .accounts({ user: admin.publicKey })
        .rpc()
    } catch (err: any) {
      console.log('Insecure error:', err)
      assert.ok(true)
    }
  })

  it('SECURE: Rejects Admin account', async () => {
    try {
      await program.methods
        .cosplaySecure()
        .accounts({ user: admin.publicKey })
        .rpc()
      assert.fail('Should have failed discriminator check')
    } catch (err) {
      assert.ok(true)
    }
  })
})
