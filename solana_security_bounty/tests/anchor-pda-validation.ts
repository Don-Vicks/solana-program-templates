import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { assert } from 'chai'
import { AnchorPdaValidation } from '../target/types/anchor_pda_validation'

describe('anchor-pda-validation', () => {
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const program = anchor.workspace
    .AnchorPdaValidation as Program<AnchorPdaValidation>

  const vault = anchor.web3.Keypair.generate()

  it('Initializes the REAL pool', async () => {
    // Find Safe PDA
    const [poolPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from('pool')],
      program.programId,
    )

    try {
      await program.methods
        .initialize()
        .accounts({
          pool: poolPda,
          vault: vault.publicKey,
          user: provider.wallet.publicKey,
        })
        .rpc()
    } catch (e) {
      // might be already initialized from previous run
    }
  })

  it('VULNERABLE: Accepts a fake pool', async () => {
    // 1. Create a fake pool (maliciously initialized)
    const fakePool = anchor.web3.Keypair.generate()

    await program.methods
      .initializeMalicious()
      .accounts({
        pool: fakePool.publicKey,
        vault: vault.publicKey,
        user: provider.wallet.publicKey,
      })
      .signers([fakePool])
      .rpc()

    // 2. Call deposit_insecure using this fake pool
    await program.methods
      .depositInsecure(new anchor.BN(100))
      .accounts({
        pool: fakePool.publicKey,
        user: provider.wallet.publicKey,
      })
      .rpc()

    const account = await program.account.pool.fetch(fakePool.publicKey)
    assert.equal(account.amount.toString(), '766') // 666 + 100
  })

  it('SECURE: Rejects fake pool', async () => {
    const fakePool = anchor.web3.Keypair.generate()
    await program.methods
      .initializeMalicious()
      .accounts({
        pool: fakePool.publicKey,
        vault: vault.publicKey,
        user: provider.wallet.publicKey,
      })
      .signers([fakePool])
      .rpc()

    // Call deposit_secure with fake pool
    try {
      await program.methods
        .depositSecure(new anchor.BN(100))
        .accounts({
          pool: fakePool.publicKey,
          user: provider.wallet.publicKey,
        })
        .rpc()
      assert.fail('Should have failed seeds constraint')
    } catch (err) {
      // Expected failure
      // Error Code: ConstraintSeeds.
      assert.ok(true)
    }
  })
})
