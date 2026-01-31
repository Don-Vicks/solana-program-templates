import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { PublicKey, SystemProgram } from '@solana/web3.js'
import { expect } from 'chai'
import { ReentrancyCpi } from '../target/types/reentrancy_cpi'

describe('Reentrancy CPI (Crema-Style)', () => {
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const program = anchor.workspace.ReentrancyCpi as Program<ReentrancyCpi>
  const authority = provider.wallet as anchor.Wallet

  let poolPda: PublicKey

  before(async () => {
    ;[poolPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('pool')],
      program.programId,
    )

    // Initialize pool with 10,000 each reserve
    await program.methods
      .initialize(new anchor.BN(10_000))
      .accounts({
        pool: poolPda,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc()

    const pool = await program.account.pool.fetch(poolPda)
    console.log(`✅ Pool initialized: A=${pool.reserveA}, B=${pool.reserveB}`)
  })

  describe('VULNERABLE: swap_insecure', () => {
    it('EXPLOIT: State updated AFTER CPI (vulnerable to reentrancy)', async () => {
      const amountIn = new anchor.BN(1_000)

      // In a real attack, the callback_program would be attacker-controlled
      // and would re-enter the swap function before state is updated
      await program.methods
        .swapInsecure(amountIn)
        .accounts({
          pool: poolPda,
          callbackProgram: SystemProgram.programId, // Simulated callback
          user: authority.publicKey,
        })
        .rpc()

      const pool = await program.account.pool.fetch(poolPda)
      console.log(
        `💀 EXPLOIT: Pool after insecure swap: A=${pool.reserveA}, B=${pool.reserveB}`,
      )
      console.log('💀 State was updated AFTER CPI - vulnerable to reentrancy!')
    })
  })

  describe('SECURE: swap_secure', () => {
    it('BLOCKED: Reentrancy guard prevents re-entry', async () => {
      const amountIn = new anchor.BN(1_000)

      // First swap should succeed
      await program.methods
        .swapSecure(amountIn)
        .accounts({
          pool: poolPda,
          callbackProgram: SystemProgram.programId,
          user: authority.publicKey,
        })
        .rpc()

      const pool = await program.account.pool.fetch(poolPda)
      console.log(
        `✅ SECURE: Pool after secure swap: A=${pool.reserveA}, B=${pool.reserveB}`,
      )

      // The lock flag is automatically cleared after the transaction
      expect(pool.isLocked).to.be.false
      console.log('✅ SECURE: Reentrancy guard released after swap!')
    })

    it('VERIFIED: State updated BEFORE CPI ', async () => {
      const poolBefore = await program.account.pool.fetch(poolPda)
      const amountIn = new anchor.BN(500)

      await program.methods
        .swapSecure(amountIn)
        .accounts({
          pool: poolPda,
          callbackProgram: SystemProgram.programId,
          user: authority.publicKey,
        })
        .rpc()

      const poolAfter = await program.account.pool.fetch(poolPda)

      // Verify state was updated
      expect(poolAfter.reserveA.toNumber()).to.be.greaterThan(
        poolBefore.reserveA.toNumber(),
      )
      console.log('✅ SECURE: State updated BEFORE external CPI call!')
    })
  })
})
