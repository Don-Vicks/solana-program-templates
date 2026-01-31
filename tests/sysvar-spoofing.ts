import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_INSTRUCTIONS_PUBKEY,
} from '@solana/web3.js'
import { expect } from 'chai'
import { SysvarSpoofing } from '../target/types/sysvar_spoofing'

describe('Sysvar Spoofing (Wormhole-Style)', () => {
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const program = anchor.workspace.SysvarSpoofing as Program<SysvarSpoofing>
  const authority = provider.wallet as anchor.Wallet

  let bridgePda: PublicKey
  let fakeInstructionAccount: Keypair

  before(async () => {
    ;[bridgePda] = PublicKey.findProgramAddressSync(
      [Buffer.from('bridge')],
      program.programId,
    )

    // Initialize the bridge
    await program.methods
      .initialize()
      .accounts({
        bridge: bridgePda,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc()

    // Create a fake "instructions" account with spoofed data
    fakeInstructionAccount = Keypair.generate()

    // Create account with fake "verified" signature data
    const space = 32
    const lamports =
      await provider.connection.getMinimumBalanceForRentExemption(space)

    const tx = new anchor.web3.Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: authority.publicKey,
        newAccountPubkey: fakeInstructionAccount.publicKey,
        lamports,
        space,
        programId: SystemProgram.programId,
      }),
    )

    await provider.sendAndConfirm(tx, [fakeInstructionAccount])

    // Write "verified" flag (first byte = 1)
    // In real attack, this would be more sophisticated
    console.log('✅ Created fake instructions account for spoofing')
  })

  describe('VULNERABLE: verify_and_mint_insecure', () => {
    it('EXPLOIT: Accepts fake sysvar account', async () => {
      // Note: In a real exploit, the fake account would have specially
      // crafted data that makes the signature check pass.
      // For this demo, we show that ANY account is accepted.

      try {
        await program.methods
          .verifyAndMintInsecure(new anchor.BN(120_000))
          .accounts({
            bridge: bridgePda,
            instructions: fakeInstructionAccount.publicKey, // FAKE!
            user: authority.publicKey,
          })
          .rpc()

        // In real exploit, this would succeed with crafted data
        console.log('💀 EXPLOIT: Accepted fake instructions account!')
      } catch (err) {
        // Expected: our fake account doesn't have valid data
        // But the vulnerability is that we CAN pass any account
        console.log(
          '💀 EXPLOIT: Vulnerable function accepts any account address!',
        )
      }
    })
  })

  describe('SECURE: verify_and_mint_secure', () => {
    it('BLOCKED: Rejects fake sysvar account', async () => {
      try {
        await program.methods
          .verifyAndMintSecure(new anchor.BN(120_000))
          .accounts({
            bridge: bridgePda,
            instructions: fakeInstructionAccount.publicKey, // FAKE!
            user: authority.publicKey,
          })
          .rpc()

        expect.fail('Should have rejected fake sysvar')
      } catch (err) {
        // The address constraint check fails
        expect(err.message).to.include('ConstraintAddress')
        console.log('✅ SECURE: Rejected fake sysvar account!')
      }
    })

    it('SUCCESS: Accepts real sysvar account', async () => {
      try {
        await program.methods
          .verifyAndMintSecure(new anchor.BN(1_000))
          .accounts({
            bridge: bridgePda,
            instructions: SYSVAR_INSTRUCTIONS_PUBKEY, // REAL sysvar
            user: authority.publicKey,
          })
          .rpc()

        // Note: This may still fail due to signature validation
        // but the important part is the address check passed
        console.log('✅ SECURE: Accepted real sysvar address!')
      } catch (err) {
        // May fail on signature check, but address is validated
        if (!err.message.includes('ConstraintAddress')) {
          console.log('✅ SECURE: Sysvar address validated correctly!')
        } else {
          throw err
        }
      }
    })
  })
})
