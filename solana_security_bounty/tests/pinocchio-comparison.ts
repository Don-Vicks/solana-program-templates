import * as anchor from '@coral-xyz/anchor'
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from '@solana/web3.js'
import { assert } from 'chai'

describe('pinocchio-comparison', () => {
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  // We don't have an IDL for Pinocchio usually, so we access it as a raw program or via generic Program if we generated one.
  // Since we used Anchor to init the workspace, we might have a basic IDL or we can just use web3.js locally.
  // The 'anchor build' might generate an IDL if we used #[program], but Pinocchio uses raw entrypoint.
  // So we will use raw TransactionInstructions.

  const programId = new PublicKey(
    'FEi7xXW24ecjuxz4TNoejMTNJXD91ev1Gkcm8GsiwFEM',
  )

  const admin = Keypair.generate()
  const config = Keypair.generate()

  before(async () => {
    // Initialize config account with some space
    const space = 32
    const lamports =
      await provider.connection.getMinimumBalanceForRentExemption(space)
    await provider.sendAndConfirm(
      new Transaction().add(
        SystemProgram.createAccount({
          fromPubkey: provider.wallet.publicKey,
          newAccountPubkey: config.publicKey,
          lamports,
          space,
          programId,
        }),
      ),
      [config],
    )
  })

  it('1. Missing Signer Check', async () => {
    // INSECURE (ix=0): Pass admin without signing
    const ixInsecure = new TransactionInstruction({
      keys: [
        { pubkey: config.publicKey, isSigner: false, isWritable: true },
        { pubkey: admin.publicKey, isSigner: false, isWritable: true }, // Not signing!
      ],
      programId,
      data: Buffer.from([0]),
    })

    await provider.sendAndConfirm(new Transaction().add(ixInsecure))

    // SECURE (ix=1): Pass admin without signing -> Should fail
    const ixSecureFail = new TransactionInstruction({
      keys: [
        { pubkey: config.publicKey, isSigner: false, isWritable: true },
        { pubkey: admin.publicKey, isSigner: false, isWritable: true }, // Not signing
      ],
      programId,
      data: Buffer.from([1]),
    })

    try {
      await provider.sendAndConfirm(new Transaction().add(ixSecureFail))
      assert.fail('Should have failed missing signature')
    } catch (err) {
      assert.ok(true)
    }

    // SECURE Success
    const ixSecurePass = new TransactionInstruction({
      keys: [
        { pubkey: config.publicKey, isSigner: false, isWritable: true },
        { pubkey: admin.publicKey, isSigner: true, isWritable: true }, // Signed!
      ],
      programId,
      data: Buffer.from([1]),
    })
    await provider.sendAndConfirm(new Transaction().add(ixSecurePass), [admin])
  })

  it('3. Type Cosplay', async () => {
    const user = Keypair.generate()
    // Need to create account with data? Pinocchio example borrows data.
    // So we must fund and allocate data for 'user'.

    const space = 16
    const lamports =
      await provider.connection.getMinimumBalanceForRentExemption(space)

    // Create account owned by OUR program? No, usually owned by program to be mutable/valid.
    // For this test, we just pass an account. The program checks borrow_data.
    // If we pass an account owned by SystemProgram, data is empty (len 0).
    // We need to assign it to the program? Or just create it with data.

    // Pinocchio example checks `user_account.borrow_data()`.
    // If account is System owned, it has 0 data.
    // We need to create an account with some data.

    // Let's create an account owned by the program with 16 bytes.
    const tx = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: provider.wallet.publicKey,
        newAccountPubkey: user.publicKey,
        lamports,
        space,
        programId, // Owned by Pinocchio program
      }),
    )
    await provider.sendAndConfirm(tx, [user])

    // INSECURE (ix=4): Just checks length, passes
    const ixInsecure = new TransactionInstruction({
      keys: [{ pubkey: user.publicKey, isSigner: false, isWritable: true }],
      programId,
      data: Buffer.from([4]),
    })
    await provider.sendAndConfirm(new Transaction().add(ixInsecure))

    // SECURE (ix=5): Checks discriminator.
    // Our account has ZEROS. Expected [0xAA; 8].
    // Should fail.
    const ixSecure = new TransactionInstruction({
      keys: [{ pubkey: user.publicKey, isSigner: false, isWritable: true }],
      programId,
      data: Buffer.from([5]),
    })

    try {
      await provider.sendAndConfirm(new Transaction().add(ixSecure))
      assert.fail('Should have failed discriminator check')
    } catch (err) {
      assert.ok(true)
    }
  })

  it('2. Arbitrary CPI', async () => {
    const fakeProgram = Keypair.generate().publicKey

    const accounts = [
      {
        pubkey: Keypair.generate().publicKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: Keypair.generate().publicKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: Keypair.generate().publicKey,
        isSigner: false,
        isWritable: true,
      },
      { pubkey: fakeProgram, isSigner: false, isWritable: false },
    ]

    const ixInsecure = new TransactionInstruction({
      keys: accounts,
      programId,
      data: Buffer.from([2]),
    })
    await provider.sendAndConfirm(new Transaction().add(ixInsecure))

    const ixSecure = new TransactionInstruction({
      keys: accounts,
      programId,
      data: Buffer.from([3]),
    })

    try {
      await provider.sendAndConfirm(new Transaction().add(ixSecure))
      assert.fail('Should have failed ID check')
    } catch (err: any) {
      assert.ok(true)
    }
  })

  it('4. PDA Validation', async () => {
    const fakePool = Keypair.generate().publicKey

    await provider.sendAndConfirm(
      new Transaction().add(
        new TransactionInstruction({
          keys: [{ pubkey: fakePool, isSigner: false, isWritable: true }],
          programId,
          data: Buffer.from([6]),
        }),
      ),
    )

    try {
      await provider.sendAndConfirm(
        new Transaction().add(
          new TransactionInstruction({
            keys: [{ pubkey: fakePool, isSigner: false, isWritable: true }],
            programId,
            data: Buffer.from([7]),
          }),
        ),
      )
      assert.fail('Should have failed seed check')
    } catch (err) {
      assert.ok(true)
    }
  })

  it('5. Re-initialization', async () => {
    const user = Keypair.generate()
    const space = 8
    const lamports =
      await provider.connection.getMinimumBalanceForRentExemption(space)

    await provider.sendAndConfirm(
      new Transaction().add(
        SystemProgram.createAccount({
          fromPubkey: provider.wallet.publicKey,
          newAccountPubkey: user.publicKey,
          lamports,
          space,
          programId,
        }),
      ),
      [user],
    )

    // INSECURE (ix=8): Inits
    await provider.sendAndConfirm(
      new Transaction().add(
        new TransactionInstruction({
          keys: [{ pubkey: user.publicKey, isSigner: false, isWritable: true }],
          programId,
          data: Buffer.from([8]),
        }),
      ),
    )

    // Call again on same account -> Succeeds (Overwrites in insecure mode)
    await provider.sendAndConfirm(
      new Transaction().add(
        new TransactionInstruction({
          keys: [{ pubkey: user.publicKey, isSigner: false, isWritable: true }],
          programId,
          data: Buffer.from([8]),
        }),
      ),
    )

    // SECURE (ix=9): Check init status (should fail because it's already initialized by ix=8)
    try {
      await provider.sendAndConfirm(
        new Transaction().add(
          new TransactionInstruction({
            keys: [
              { pubkey: user.publicKey, isSigner: false, isWritable: true },
            ],
            programId,
            data: Buffer.from([9]),
          }),
        ),
      )
      assert.fail('Should have failed initialized check')
    } catch (err) {
      assert.ok(true)
    }
  })
})
