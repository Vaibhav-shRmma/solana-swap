import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaSwap } from "../target/types/solana_swap";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  getAccount,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";

describe ("solana-swap", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaSwap as Program<SolanaSwap>;
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  let mintA: anchor.web3.PublicKey;
  let mintB: anchor.web3.PublicKey;
  let poolTokenA: anchor.web3.PublicKey;
  let poolTokenB: anchor.web3.PublicKey;
  let userTokenA: anchor.web3.PublicKey;
  let userTokenB: anchor.web3.PublicKey;
  let poolPda: anchor.web3.PublicKey;

  before(async () => {
    mintA = await createMint(
      connection,
      wallet.payer,
      wallet.publicKey,
      null,
      6
    );

    mintB = await createMint(
      connection,
      wallet.payer,
      wallet.publicKey,
      null,
      6
    );

    [poolPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("pool")],
      program.programId
    );

    poolTokenA = (await getOrCreateAssociatedTokenAccount(
      connection,
      wallet.payer,
      mintA,
      poolPda,
      true
    )).address;

    poolTokenB = (await getOrCreateAssociatedTokenAccount(
      connection,
      wallet.payer,
      mintB,
      poolPda,
      true
    )).address;

    userTokenA = (await getOrCreateAssociatedTokenAccount(
      connection,
      wallet.payer,
      mintA,
      wallet.publicKey
    )).address;

    userTokenB = (await getOrCreateAssociatedTokenAccount(
      connection,
      wallet.payer,
      mintB,
      wallet.publicKey
    )).address;

    await mintTo(
      connection,
      wallet.payer,
      mintA,
      userTokenA,
      wallet.publicKey,
      1_000_000_000
    );

    await mintTo(
      connection,
      wallet.payer,
      mintB,
      userTokenB,
      wallet.publicKey,
      1_000_000_000
    );

    await mintTo(connection, wallet.payer, mintA, poolTokenA, wallet.payer, 500_000_000);
    await mintTo(connection, wallet.payer, mintB, poolTokenB, wallet.payer, 500_000_000);
  });


  //TEST 1
  it("Initialize the pool", async ()=> {
    await program.methods.initializePool(
      new anchor.BN(3),
      new anchor.BN(1000)
    )
    .accounts({
      authority: wallet.publicKey,
      tokenAAccount: poolTokenA,
      tokenBAccount: poolTokenB,
    })
    .rpc();

    const pool = await program.account.pool.fetch(poolPda);
    assert.equal(pool.feeNumerator.toNumber(), 3);
    assert.equal(pool.feeDenominator.toNumber(), 1000);
    assert.ok(pool.authority.equals(wallet.publicKey));
    console.log("pool initialized with 0.3% fee")
  });

  //TEST 2
  it("Adds Liquidity to the pool", async () => {
    const poolABefore = await getAccount(connection, poolTokenA);

    await program.methods.addLiquidity(
      new anchor.BN(100_000_000),
      new anchor.BN(100_000_000)
    ) 
    .accounts({
      user: wallet.publicKey,
      userTokenA,
      userTokenB,
      poolTokenA,
      poolTokenB,
      mintA,
      mintB,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .rpc();

  const poolAAfter = await getAccount(connection, poolTokenA);
  const deposited = Number(poolAAfter.amount) - Number(poolABefore.amount);
  assert.equal(deposited, 100_000_000);
  console.log(`added liquidity : ${deposited} Token A deposited`);
  });

  // Test 3: Swap Token A → Token B
  it("Swaps Token A for Token B", async () => {
    const userBBefore = await getAccount(connection, userTokenB);

    await program.methods
      .swapAToB(new anchor.BN(50_000_000))
      .accounts({
        user: wallet.publicKey,
        userTokenA,
        userTokenB,
        poolTokenA,
        poolTokenB,
        mintA,
        mintB,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const userBAfter = await getAccount(connection, userTokenB);
    const received = Number(userBAfter.amount) - Number(userBBefore.amount);

    assert.isAbove(received, 0);
    console.log(`Swapped 50 Token A → received ${received} Token B`);
  });
});