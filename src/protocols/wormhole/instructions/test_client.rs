use solana_sdk::{instruction::{Instruction, AccountMeta},
    signer::keypair::read_keypair_file, 
    transaction::{Transaction, VersionedTransaction}, 
    pubkey::Pubkey, 
    signer::{keypair::Keypair, Signer}, 
    signers::Signers, 
    native_token::LAMPORTS_PER_SOL,
    system_instruction::transfer,
};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct SystemProgramTransferInstructionData {
    // pub from: Pubkey,
    // pub to: Pubkey,
    pub descriminator: u32,
    pub lamports: u64,
}


fn main() {
    println!("Hello, Solana Rust SDK!");

    let keypair = read_keypair_file("~/.config/solana/id.json").unwrap();
    let my_pubkey: Pubkey = keypair.try_pubkey().unwrap();
    let random_pubkey: Pubkey = Pubkey::new_unique();
    println!("My pubkey: {}", my_pubkey);
    println!("Random pubkey: {}", random_pubkey);

    let rpc = RpcClient::new("https://api.devnet.solana.com").to_string();
    // rpc.request_airdrop(my_pubkey, 100000000).unwrap();

    let system_program_id: Pubkey = Pubkey::default();
    //let instruction_data: &[u8; 12] = &[2, 0, 0, 0, 42, 0, 0, 0, 0, 0, 0, 0];
     
    let instruction_data_struct = SystemProgramTransferInstructionData {
        descriminator: 2,
        lamports: (0.1 * LAMPORTS_PER_SOL as f64) as u64,
        //lamports: 0.1 * LAMPORTS_PER_SOL as u64,
    };

    let mut instruction_data: Vec<u8> = Vec::new();
    instruction_data_struct.serialize(&mut instruction_data).unwrap();

    let instruction_accounts: Vec<AccountMeta> = vec![
    AccountMeta { pubkey: my_pubkey, is_signer: true, is_writable: true },
    AccountMeta { pubkey: random_pubkey, is_signer: false, is_writable: true },
    ];

    let ix = Instruction::new_with_bytes(system_program_id, instruction_accounts, instruction_data);
    //let transfer_ix = transfer(&my_pubkey, &random_pubkey, 0.1 * LAMPORTS_PER_SOL as u64);
    
    let signers : Signers = &[&keypair];
    let blockhash = rpc.get_latest_blockhash().unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&my_pubkey),
        &signers,
        blockhash
    );

    //Instead of this it, it's better to use rpc.send_transaction_with_config
    //let sx = rpc.send_and_confirm_transaction(&tx);

    let mut config = RpcSendTransactionConfig::default();
    config.skip_preflight = true;
    let sx = rpc.send_transaction_with_config(&tx, config).unwrap();
    //let sx = rpc.send_and_confirm_transaction_with_spinner(&tx).unwrap();
    println!("Signature: {}", sx);

    //println!("{:?}", sx); 
    

 
}
