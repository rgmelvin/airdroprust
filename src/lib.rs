use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signer, read_keypair_file},
    transaction::Transaction,
    message::Message,
};
use solana_program::{
    pubkey::Pubkey,
    system_instruction::transfer,
    hash::hash,
    system_program,
};
use std::{
    io::{self, BufRead},
    str::FromStr,
};
use bs58;


// Connect to DevNet
const RPC_URL: &str = "https://api.devnet.solana.com";

mod programs; // <--- uses src/programs/mod.rs and src/programs/Turbin3_prereq.rs


#[cfg(test)]
mod tests {
    use super::*;
    use crate::programs::Turbin3_prereq::{
        Turbin3PrereqProgram, CompleteArgs, UpdateArgs
    };

    // KEY GENERATION
    // ------------------------------------------------------------------------------------
    // cargo test keygen -- --nocapture

    #[test]
    fn keygen() {
        // Create a new keypair
        let kp = Keypair::new();

        // Print the newly generated public key
        println!("You've generated a new Solana wallet: {}", kp.pubkey().to_string());
        println!("");
        println!("To save your wallet, copy and paste the following into a JSON file:");
        
        // keypair.to_bytes() is a [64; 1] array for the full secret key data
        println!("{:?}", kp.to_bytes());
    } 

    // BASE58 -> WALLET FILE (BYTE ARRAY)
    // ------------------------------------------------------------------------------------
    // cargo test base58_to_wallet -- --nocapture
    #[test]
    fn base58_to_wallet() {
        println!("Input your private key as base58:");

        // Read one line from stdin
        let stdin = io::stdin();
        let base58_str = stdin.lock().lines().next().unwrap().unwrap();

        println!("Your wallet file is:");

        // Decode using bs58
        let wallet_bytes = bs58::decode(base58_str.trim()).into_vec().unwrap();
        println!("{:?}", wallet_bytes);
    }

    // WALLET FILE (BYTE ARRAY) -> BASE58
    // ----------------------------------------------------------------------------------
    // cargo test wallet_to_base58 -- --nocapture
    #[test]
    fn wallet_to_base58() {
        println!("Input your private key as a wallet file byte array:");

        let stdin = io::stdin();
        let line = stdin.lock().lines().next().unwrap().unwrap();

        // Remove brackets if present and split by comma
        let trimmed = line.trim().trim_start_matches('[').trim_end_matches(']');
        let wallet_vec = trimmed
            .split(',')
            .map(|s| s.trim().parse::<u8>().unwrap())
            .collect::<Vec<u8>>();

        let base58_str = bs58::encode(wallet_vec).into_string();
        println!("Your private key in base58 is:\n{}", base58_str);
    }
    
    // AIRDROP
    // ---------------------------------------------------------------------------------
    // cargo test airdrop -- --nocapture
    #[test] 
    fn airdrop() {
        // Use the dev-wallet.json from keygen
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find dev-wallet.json");

        // Connect to devnet
        let client = RpcClient::new(RPC_URL.to_string());

        // Airdrop 2 SOL = 2_000_000_000 lamports
        match client.request_airdrop(&keypair.pubkey(), 2_000_000_000u64) {
            Ok(sig) => {
                println!("Success! Check out your TX here:");
                println!("https://explorer.solana.com/tx/{}?cluster=devnet",
                sig.to_string());
            }
            Err(e) => eprintln!("Oops, something went wrong: {}", e)
        }
    } 
    
    // TRANSFER SOME SOL
    // ---------------------------------------------------------------------------------
    // cargo test transfer_sol -- --nocapture
    #[test]
    fn transfer_sol() {
        //dev wallet private key
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find def-wallet.json");
        let pubkey =keypair.pubkey();

        // Sign a message just for demonstration purposes
        let message_bytes = b"I verify my solana Keypair!";
        let sig = keypair.sign_message(message_bytes);
        let sig_hashed = hash(sig.as_ref());

        // Verify the signature , using the default implementation.
        let verified = sig.verify(&pubkey.to_bytes(), &sig_hashed.to_bytes());
        if verified {
            println!("Signature verified");
        } else {
            println!("Verification failed");
        }

        // Define the destination (your Turbin3 public key)
        let to_pubkey = Pubkey::from_str("HrK9NkuGnVnu6TsEvRfdDhsnTBEH89VtGnANyTCUAyXm").unwrap();

        // Connect to Solana devnet
        let rpc_client = RpcClient::new(RPC_URL);

        // Get a recent blockhash for the transaction
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        // Transfer 0.1 SOL to your Turbin3 wallet address on Solana devnet
        let transaction = Transaction::new_signed_with_payer(
            &[transfer(
                &keypair.pubkey(),
                &to_pubkey,
                1_000_000
            )],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash
        );

        // Send and confirm transaction
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        println!("Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet",
        signature
        );
    }

    // EMPTY THE DEV WALLET
    // ------------------------------------------------------------------------------------------------
    // cargo test empty_wallet -- --nocapture
    #[test]
    fn empty_wallet() {
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find def-wallet.json");
        let rpc_client = RpcClient::new(RPC_URL);

        // Get balance of dev wallet
        let balance = rpc_client
            .get_balance(&keypair.pubkey())
            .expect("Failed to get balance");

        println!("Dev wallet balance is {} lamports.", balance);
        if balance == 0 {
            println!("No lamports to transfer!");
            return;
        }

        // Destination wallet (your Turbin3 wallet)
        let to_pubkey = Pubkey::from_str("HrK9NkuGnVnu6TsEvRfdDhsnTBEH89VtGnANyTCUAyXm").unwrap();

        // Create a mock transaction to estimate fees
        let blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get blockhash");
        let message = Message::new_with_blockhash(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance)],
            Some(&keypair.pubkey()),
            &blockhash,
        );
        let fee = rpc_client
            .get_fee_for_message(&message)
            .expect("Failed to get fee for message");

        // If fee is greater than balance, we can't send
        if fee > balance {
            println!("Balance ({}) is not enought to cover fee ({}).", balance, fee);
            return;
        }

        let final_amount = balance - fee;
        println!("Sending {} lamports to {}. (Fee = {})", final_amount, to_pubkey, fee);

        // Create and send the real transaction
        let tx = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, final_amount)],
            Some(&keypair.pubkey()),
            &[&keypair],
            blockhash,
        );
        let sig = rpc_client
            .send_and_confirm_transaction(&tx)
            .expect("Failed to send transaction");

        println!("success! Check out your TX here:\nhttps://explorer.solana.com/tx/{}?cluster=devnet", sig);
    }

    // ENROLL (USING THE IDLGEN-GENERATED CODE)
    // -------------------------------------------------------------------------------------------------------
    // cargo test enroll -- --nocapture
    #[test]
    fn enroll() {
        // Read the Turbine3-wallet.json
        let signer_keypair = read_keypair_file("Turbin3-wallet.json")
            .expect("Couldn't find Turbin3-wallet.json");
        let signer_pubkey = signer_keypair.pubkey();

        // Connect to devnet
        let rpc_client = RpcClient::new(RPC_URL.to_string());
        let blockhash = rpc_client
            .get_latest_blockhash()
            .expect("Failed to get blockhash");

        // Derive the 'prereq' PDA. Seeds = ["prereq", signer_pubkey])
        let prereq_pda = Turbin3PrereqProgram::derive_program_address(&[b"prereq", signer_pubkey.to_bytes().as_ref()]);
        println!("Derived prereq PDA: {}", prereq_pda);

        // Pass Github username as "bytes" to "complete"
        let args = CompleteArgs {
            github: b"rgmelvin".to_vec(),
        };

        // Invoke the complete function which has 3 accounts: signer, prereq, system_program
        let transaction = Turbin3PrereqProgram::complete(
            &[
                &signer_pubkey,
                &prereq_pda,
                &system_program::id(),
            ],
            &args,
            Some(&signer_pubkey), // fee payer
            &[&signer_keypair],   // signers
            blockhash
        );

        // Send the transaction
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        // Print the transaction:
        println!("Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet", signature);
    }
}
