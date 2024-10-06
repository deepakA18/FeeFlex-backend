import { Connection, Keypair, PublicKey, SystemProgram, Transaction, TransactionInstruction } from "@solana/web3.js";
import { TokenSwap, TOKEN_SWAP_PROGRAM_ID, TokenSwapLayout, CurveType } from "@solana/spl-token-swap";
import { Wallet, web3 } from "@coral-xyz/anchor";
import * as fs from 'fs';
import * as token from '@solana/spl-token-swap';
function loadkeypair(filename: string) : Keypair {
    const secret = JSON.parse(fs.readFileSync(filename, 'utf8').toString()) as number[]
    const secretKey = Uint8Array.from(secret)
    return Keypair.fromSecretKey(secretKey)
    
}

async function getTokenAccountCreationInstruction(mint: PublicKey, swapAuthority: PublicKey, payer: PublicKey): Promise<[PublicKey, TransactionInstruction]> {

    let tokenAccountAddress = await token.getAssociatedTokenAddress(
        mint,
        swapAuthority,
        true
    )

    const tokenAccountIntruction = await token.createAssociatedTokenAccountInstruction(
        tokenAccountAddress,
        payer,
        swapAuthority,
        payer,
        mint
    )
    return [tokenAccountAddress, tokenAccountIntruction];
}




async function main() {
    
    const connection = new Connection("https://api.devnet.solana.com");
    const Wallet = loadkeypair('SwkT8E3LTRiZ9iiFiycL3VBD3HEYrr3akyMPUQt4YfN.json');
    const transaction = new Transaction();
    const tokenSwapStateAccount = Keypair.generate();
    const rent = await connection.getMinimumBalanceForRentExemption(TokenSwapLayout.span);
    const tokenSwapStateAccountInstruction = SystemProgram.createAccount({
        fromPubkey: Wallet.publicKey,
        newAccountPubkey: tokenSwapStateAccount.publicKey,
        lamports: rent,
        space: TokenSwapLayout.span,
        programId: TOKEN_SWAP_PROGRAM_ID,
    })
    transaction.add(tokenSwapStateAccountInstruction)

    const [swapAuthority, bump] = await PublicKey.findProgramAddressSync(
        [tokenSwapStateAccount.publicKey.toBuffer()],
        TOKEN_SWAP_PROGRAM_ID
    )

    const tokenAmint = new PublicKey("AT9AQv5uhZWk7QiesD538XyBY31GEtG2Ps7zdo7naWP6");
    const tokenBmint = new PublicKey("BThX9vtQfKD79z8bvupCmojyYAXj8XxmztbFWfRJ7QRL");
    const [tokenATokenAccount, tokenAAccountCreationInstruction] = await getTokenAccountCreationInstruction(tokenAmint, swapAuthority, Wallet.publicKey);
    const [tokenBtokenAccount, tokenBAccountCreationInstruction] = await getTokenAccountCreationInstruction(tokenBmint, swapAuthority, Wallet.publicKey);
    transaction.add(tokenAAccountCreationInstruction, tokenBAccountCreationInstruction)

    const signature = await web3.sendAndConfirmTransaction(connection, transaction, [Wallet]);
    console.log('Transaction Signature:', signature);




    const poolTokenMint = new PublicKey("LPazMPCfSWKhfDcEyEYhtR47VupFRxDE8mqqfrR9jVF.json")
    const tokenAccountPool = Keypair.generate();
    const poolTokenAccountsize = 165; // Define the size of the pool token account
    const poolAccountRent = await connection.getMinimumBalanceForRentExemption(poolTokenAccountsize);   
    const createTokenAccountPoolInstruction = SystemProgram.createAccount({
        fromPubkey: Wallet.publicKey,
        newAccountPubkey: tokenAccountPool.publicKey,
        lamports: poolAccountRent,
        space: 165,
        programId: TOKEN_SWAP_PROGRAM_ID,
    })
    
    const initializTokenAccountPoolInstruction =  token.createInitializeAccountInstruction(
        tokenAccountPool.publicKey,
        poolTokenMint,
        Wallet.publicKey
    )
    transaction.add(createTokenAccountPoolInstruction, initializTokenAccountPoolInstruction)

    const feeOwner = new PublicKey("LPbxNx3UTnQztaU2uFq5Z5nVtMpjm2v8iPxwqjGC4ji")

    const [tokenFeeAccountAddress, tokenFeeAccountCreationInstruction] = await getTokenAccountCreationInstruction(
        poolTokenMint, feeOwner, Wallet.publicKey);
    transaction.add(tokenFeeAccountCreationInstruction)

     
    
    const [poolTokenAccount, poolTokenAccountCreationInstruction] = await getTokenAccountCreationInstruction(
        poolTokenMint, feeOwner, Wallet.publicKey);

    const tokenFeeAccountInstruction = token.createAssociatedTokenAccountInstruction(
        Wallet.publicKey,
        feeOwner,
        tokenFeeAccountAddress,
        poolTokenMint
    );


    const tokenSwapInitInstruction = TokenSwap.createInitSwapInstruction(  
        tokenSwapStateAccount,
        Wallet.publicKey,
        swapAuthority,
        tokenATokenAccount,
        tokenBtokenAccount,
        tokenAccountPool.publicKey,
        tokenFeeAccountAddress,
        Wallet.publicKey,
        TOKEN_SWAP_PROGRAM_ID,
        token.TOKEN_SWAP_PROGRAM_ID,
        0, // fee numerator
        1, // fee denominator
        0, // owner trade fee numerator
        1, // owner trade fee denominator
        0, // owner withdraw fee numerator
        1, // owner withdraw fee denominator
        0, // host fee numerator
        1, // host fee denominator
        CurveType.ConstantProduct,
    )
    transaction.add(tokenSwapInitInstruction)

    const signature2 = await web3.sendAndConfirmTransaction(connection, transaction, [Wallet, tokenAccountPool]);
    console.log('Transaction Signature:', signature2);

     


}

    
    
    

