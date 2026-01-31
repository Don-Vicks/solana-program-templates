import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import {
  createAccount,
  createMint,
  getAccount,
  mintTo,
} from '@solana/spl-token'
import { PublicKey } from '@solana/web3.js'
import { expect } from 'chai'
import { InfiniteMint } from '../target/types/infinite_mint'

describe('Infinite Mint (Cashio-Style)', () => {
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const program = anchor.workspace.InfiniteMint as Program<InfiniteMint>
  const authority = provider.wallet as anchor.Wallet

  let stablecoinMint: PublicKey
  let approvedCollateralMint: PublicKey
  let fakeCollateralMint: PublicKey
  let configPda: PublicKey

  let userApprovedCollateralAccount: PublicKey
  let userFakeCollateralAccount: PublicKey
  let userStablecoinAccount: PublicKey

  before(async () => {
    // Create the stablecoin mint (authority will be config PDA)
    ;[configPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('config')],
      program.programId,
    )

    stablecoinMint = await createMint(
      provider.connection,
      authority.payer,
      configPda, // Mint authority is config PDA
      null,
      6,
    )

    // Create approved collateral (e.g., USDC)
    approvedCollateralMint = await createMint(
      provider.connection,
      authority.payer,
      authority.publicKey,
      null,
      6,
    )

    // Create fake collateral (attacker's worthless token)
    fakeCollateralMint = await createMint(
      provider.connection,
      authority.payer,
      authority.publicKey,
      null,
      6,
    )

    // Create user token accounts
    userApprovedCollateralAccount = await createAccount(
      provider.connection,
      authority.payer,
      approvedCollateralMint,
      authority.publicKey,
    )

    userFakeCollateralAccount = await createAccount(
      provider.connection,
      authority.payer,
      fakeCollateralMint,
      authority.publicKey,
    )

    userStablecoinAccount = await createAccount(
      provider.connection,
      authority.payer,
      stablecoinMint,
      authority.publicKey,
    )

    // Mint some approved collateral to user
    await mintTo(
      provider.connection,
      authority.payer,
      approvedCollateralMint,
      userApprovedCollateralAccount,
      authority.publicKey,
      1_000_000, // 1 USDC
    )

    // Mint fake collateral to user (attacker creates worthless tokens)
    await mintTo(
      provider.connection,
      authority.payer,
      fakeCollateralMint,
      userFakeCollateralAccount,
      authority.publicKey,
      1_000_000_000, // 1000 fake tokens
    )

    // Initialize the protocol
    await program.methods
      .initialize(approvedCollateralMint)
      .accounts({
        stablecoinMint: stablecoinMint,
        authority: authority.publicKey,
      })
      .rpc()
  })

  describe('VULNERABLE: mint_insecure', () => {
    it('EXPLOIT: Accepts fake collateral and mints stablecoins', async () => {
      const mintAmount = new anchor.BN(1_000_000)

      // Attacker uses FAKE collateral to mint real stablecoins!
      await program.methods
        .mintInsecure(mintAmount)
        .accounts({
          userCollateral: userFakeCollateralAccount, // FAKE TOKEN!
          stablecoinMint: stablecoinMint,
          userStablecoin: userStablecoinAccount,
          user: authority.publicKey,
        })
        .rpc()

      const stablecoinBalance = await getAccount(
        provider.connection,
        userStablecoinAccount,
      )

      // Attacker successfully minted stablecoins with worthless collateral
      expect(Number(stablecoinBalance.amount)).to.equal(1_000_000)
      console.log('💀 EXPLOIT: Minted 1M stablecoins with FAKE collateral!')
    })
  })

  describe('SECURE: mint_secure', () => {
    it('BLOCKED: Rejects fake collateral', async () => {
      const mintAmount = new anchor.BN(500_000)

      try {
        await program.methods
          .mintSecure(mintAmount)
          .accounts({
            userCollateral: userFakeCollateralAccount, // FAKE TOKEN!
            stablecoinMint: stablecoinMint,
            userStablecoin: userStablecoinAccount,
            user: authority.publicKey,
          })
          .rpc()

        expect.fail('Should have rejected fake collateral')
      } catch (err) {
        expect(err.message).to.include('InvalidCollateral')
        console.log('✅ SECURE: Rejected fake collateral!')
      }
    })

    it('SUCCESS: Accepts approved collateral', async () => {
      const mintAmount = new anchor.BN(500_000)

      await program.methods
        .mintSecure(mintAmount)
        .accounts({
          userCollateral: userApprovedCollateralAccount, // REAL collateral
          stablecoinMint: stablecoinMint,
          userStablecoin: userStablecoinAccount,
          user: authority.publicKey,
        })
        .rpc()

      const stablecoinBalance = await getAccount(
        provider.connection,
        userStablecoinAccount,
      )

      // 1M from exploit + 500K from secure mint
      expect(Number(stablecoinBalance.amount)).to.equal(1_500_000)
      console.log('✅ SECURE: Minted with valid collateral!')
    })
  })
})
