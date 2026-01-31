import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { Keypair, PublicKey } from '@solana/web3.js'
import { expect } from 'chai'
import { AccessControl } from '../target/types/access_control'

describe('Access Control (Raydium-Style)', () => {
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const program = anchor.workspace.AccessControl as Program<AccessControl>
  const admin = provider.wallet as anchor.Wallet
  const attacker = Keypair.generate()

  let vaultPda: PublicKey

  before(async () => {
    ;[vaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('vault')],
      program.programId,
    )

    // Initialize vault with admin
    await program.methods
      .initialize()
      .accounts({
        admin: admin.publicKey,
      })
      .rpc()

    // Deposit funds to vault
    await program.methods
      .deposit(new anchor.BN(100_000))
      .accounts({
        depositor: admin.publicKey,
      })
      .rpc()

    const vault = await program.account.vault.fetch(vaultPda)
    console.log(`✅ Vault initialized with ${vault.balance} balance`)
  })

  describe('VULNERABLE: withdraw_insecure', () => {
    it('EXPLOIT: Admin key can drain instantly', async () => {
      const withdrawAmount = new anchor.BN(50_000)

      // If admin key is compromised, attacker drains immediately
      await program.methods
        .withdrawInsecure(withdrawAmount)
        .accounts({
          signer: admin.publicKey,
        })
        .rpc()

      const vault = await program.account.vault.fetch(vaultPda)
      expect(vault.balance.toNumber()).to.equal(50_000)
      console.log('💀 EXPLOIT: 50,000 withdrawn INSTANTLY - no timelock!')
    })

    it('BLOCKED: Non-admin cannot withdraw', async () => {
      // Airdrop to attacker for fees
      await provider.connection.requestAirdrop(attacker.publicKey, 1e9)
      await new Promise((resolve) => setTimeout(resolve, 1000))

      try {
        await program.methods
          .withdrawInsecure(new anchor.BN(1_000))
          .accounts({
            signer: attacker.publicKey,
          })
          .signers([attacker])
          .rpc()

        expect.fail('Non-admin should not withdraw')
      } catch (err) {
        expect(err.message).to.include('Unauthorized')
        console.log(
          '✅ Non-admin rejected (but if key is compromised, game over)',
        )
      }
    })
  })

  describe('SECURE: Timelock pattern', () => {
    it('Step 1: Initiate withdrawal (starts timelock)', async () => {
      const withdrawAmount = new anchor.BN(25_000)
      const recipient = admin.publicKey

      await program.methods
        .initiateWithdrawalSecure(withdrawAmount, recipient)
        .accounts({
          signer: admin.publicKey,
        })
        .rpc()

      const vault = await program.account.vault.fetch(vaultPda)
      expect(vault.pendingWithdrawal).to.not.be.null
      console.log(
        `✅ SECURE: Withdrawal initiated, locked until slot ${vault.pendingWithdrawal.executableAfter}`,
      )
    })

    it('Step 2: Cannot execute before timelock expires', async () => {
      try {
        await program.methods.executeWithdrawalSecure().rpc()

        expect.fail('Should not execute before timelock')
      } catch (err) {
        expect(err.message).to.include('TimelockNotExpired')
        console.log('✅ SECURE: Blocked - timelock not expired!')
      }
    })

    it('Step 3: Admin can cancel suspicious withdrawal', async () => {
      await program.methods
        .cancelWithdrawalSecure()
        .accounts({
          signer: admin.publicKey,
        })
        .rpc()

      const vault = await program.account.vault.fetch(vaultPda)
      expect(vault.pendingWithdrawal).to.be.null
      console.log('✅ SECURE: Pending withdrawal cancelled!')
    })

    it('VERIFIED: Balance unchanged after cancelled withdrawal', async () => {
      const vault = await program.account.vault.fetch(vaultPda)
      expect(vault.balance.toNumber()).to.equal(50_000)
      console.log('✅ SECURE: Funds safe, timelock gave time to react!')
    })
  })
})
