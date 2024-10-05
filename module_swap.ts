import { Transaction, Keypair, SystemProgram, PublicKey, TransactionInstruction} from "@solana/web3.js";
import { TokenSwap, TOKEN_SWAP_PROGRAM_ID, TokenSwapLayout } from "@solana/spl-token-swap";
import * as token from "@solana/spl-token-swap"
import fs from 'fs';

function loadkeypair(filename: string) : Keypair{
    const secret = JSON.parse(fs.readFileSync(filename).toString()) as number[]
    const secretKey = Uint8Array.from(secret)
    return Keypair.fromSecretKey(secretKey)
    
} 

async function getTokenAccountCreationInstruction(mint: PublicKey, swapAuthority: PublicKey, payer: PublicKey): Promise<[PublicKey,TransactionInstruction]>{
    let tokenAccountAddress = await token.getAssociatedTokenAddress(
        mint, //mint
        swapAuthority, //owner
        true //allow owner
    
    let tokenAccountInstruction = await token.createAssociatedTokenAccountInstruction(
        payer, //payer
        tokenAccountAddress
        swapAuthority, //owner
        mint //allow owner  
        
    return [tokenAcountAddress, tokenAccountInstruction];
        
}

async function main() {
    const connection = new Connection("https://api.devnet.solana.com");
    const wallet = loadkeypair( 'sasBM6wN3hGDJ6SDwgH9texQUHufQ5CWafPCbKJGH8X.json')
    const transaction = new Transaction()
    const tokenSwapStateAccount = loadkeypair("TSSEksYP6NM1t1AhsKXRwWZ7RuhbFoT4JYydV8EsKk4.json")
    const rent =  await TokenSwap.getMinBalanceRentForExemptTokenSwap(connect)
    const tokenSwapStateAccountCreationInstruction = SystemProgram.createAccount({
        newAccountPubkey: tokenSwapStateAccount.publicKey,
        fromPubkey: wallet.publicKey,
        lamports : rent
        space: TokenSwapLayout.span,
        programId: TOKEN_SWAP_PROGRAM_ID
    })
    transaction.add(tokenSwapStateAccountCreationInstruction)


    
    const [swapAuthority, bump] = await Web3.publicKey.findProgramAddress(
        [tokenSwapStateAccount.publicKey.toBuffer()],
        TOKEN_SWAP_PROGRAM_ID,
    )

    console.log("swap authority: " + swapAuthority.toBase58())


    const tokenAMint = new PublicKey("A9xDVxswS7RfRyZkYnCxzghBQnqYVHqBd9rbMXWqLCBC.json")
    const tokenBMint = new PublicKey("B5ChTurqKFTMDSV4eVodjapageWcNRcwSvDfzYt1pCTa.json")   
    const [tokenATokenAccount,tokenAci] = await getTokenAccountCreationInstruction(tokenAMint, swapAuthority, wallet.publicKey);
    const [tokenBTokenAccount,tokenBci] = await getTokenAccountCreationInstruction(tokenBMint, swapAuthority, wallet.publicKey); 
    
    

    const tokenSwapInitSwapInstruction = TokenSwap.createInitSwapInstruction(
        tokenSwapStateAccount.publicKey,
        swapAuthority,
        tokenATokenAccount,
        tokenBTokenAccount
    )

    
}
main();


