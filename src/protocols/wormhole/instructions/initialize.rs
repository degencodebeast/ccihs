use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{Keypair, read_keypair_file},
        system_program,
        instruction::{Instruction, AccountMeta},
        transaction::Transaction,
        sysvar::{clock, rent},
    },
    solana_client::rpc_client::RpcClient,
    Client, Program,
    Cluster,
};
use anyhow::Result;
use std::rc::Rc;
use std::str::FromStr;
use std::fmt::Debug;
use std::vec;
use log::{warn, info};

use crate::protocols::wormhole::state::{
    SenderConfig,
    RedeemerConfig,
    InboundTokenBridgeAddresses,
    OutboundTokenBridgeAddresses,
    WormholeAddresses,
    GeneralMessageConfig,
    WormholeEmitter,
    SEED_PREFIX_SENT,
};
use crate::protocols::wormhole::config::WormholeClientConfig;
use wormhole_anchor_sdk::{wormhole, token_bridge};
use crate::protocols::wormhole::config::{WormholeClient, WormholeClientConfig};

impl WormholeClient {
    pub fn new(url: &str, payer: Keypair, program_id: Pubkey, wormhole_client_config: WormholeClientConfig, commitment: CommitmentConfig) -> Result<Self> {
        let payer = Rc::new(payer);
        let client = Client::new_with_options(
            url.to_string(),
            payer.clone(),
            commitment,
        );
        let program = client.program(program_id)?;

        Ok(Self { program, payer, config: wormhole_client_config })
    }

    pub async fn initialize(
        &self
    ) -> Result<()> {
        let owner = self.payer.pubkey();

        let relayer_fee = self.config.relayer_fee;
        let relayer_fee_precision = self.config.relayer_fee_precision;

        // 1. Derive PDAs
        let (general_message_config_pda, general_message_config_bump) = Pubkey::find_program_address(
            &[b"general_message_config"],
            &self.program.id(),
        );

        // Derive sender config PDA
        let (token_sender_config_pda, token_sender_bump) = Pubkey::find_program_address(
        &[SenderConfig::SEED_PREFIX],
        &self.program.id(),
        );

       // Derive redeemer config PDA
       let (redeemer_config_pda, token_redeemer_bump) = Pubkey::find_program_address(
            &[RedeemerConfig::SEED_PREFIX],
            &self.program.id(),
        );

        // 2. Create config data structures 
        let wormhole_addresses = WormholeAddresses {
            bridge: Pubkey::from_str("wormhole_bridge_address")?,
            fee_collector: Pubkey::from_str("wormhole_fee_collector_address")?,
            sequence: Pubkey::from_str("wormhole_sequence_address")?,
        };

        let general_message_config = GeneralMessageConfig {
            owner: self.payer.pubkey(),
            wormhole: wormhole_addresses,
            batch_id: 0,
            finality: 1,
        };

        // Create the OutboundTokenBridgeAddresses structure
        let outbound_token_bridge_addresses = OutboundTokenBridgeAddresses {
            config: Pubkey::from_str("token_bridge_config_address")?,
            authority_signer: Pubkey::from_str("authority_signer_address")?,
            custody_signer: Pubkey::from_str("custody_signer_address")?,
            emitter: Pubkey::from_str("token_bridge_emitter_address")?,
            sequence: Pubkey::from_str("token_bridge_sequence_address")?,
            wormhole_bridge: Pubkey::from_str("wormhole_bridge_address")?,
            wormhole_fee_collector: Pubkey::from_str("wormhole_fee_collector_address")?,
        };

        // Create the SenderConfig data
        let token_sender_config = SenderConfig {
            owner: self.payer.pubkey(),
            bump: token_sender_bump,
            token_bridge: outbound_token_bridge_addresses,
            finality: 1, // Set appropriate finality level (1 for Confirmed)
        };
        
        let inbound_token_bridge_addresses = InboundTokenBridgeAddresses {
            config: Pubkey::from_str("token_bridge_config_address")?,
            custody_signer: Pubkey::from_str("custody_signer_address")?,
            mint_authority: Pubkey::from_str("mint_authority_address")?,
        };

        let token_redeemer_config = RedeemerConfig {
            owner: self.payer.pubkey(),
            bump: token_redeemer_bump,
            token_bridge: inbound_token_bridge_addresses,
            //use the wormhole config to fill these values
            relayer_fee,
            relayer_fee_precision,
        };

        let wormhole_program = Pubkey::from_str("wormhole_program_id_here")?;

        let token_bridge_program = Pubkey::from_str("token_bridge_program_id_here")?;

        // let (wormhole_emitter, _) = Pubkey::find_program_address(
        //     &[b"wormhole_emitter"],
        //     &self.program.id(),
        // );

        let (wormhole_emitter_pda, wormhole_emitter_bump) = Pubkey::find_program_address(
            &[WormholeEmitter::SEED_PREFIX],
            &self.program.id(),
        );

            
        let (wormhole_message_pda, wormhole_message_bump) = Pubkey::find_program_address(
            &[
                SEED_PREFIX_SENT,
                &wormhole::INITIAL_SEQUENCE.to_le_bytes()[..],
            ],
            &self.program.id(),
        );

        // 3. Create account creation instructions
        let create_account_ixs = vec![
            system_program::create_account(
                    &self.payer.pubkey(),
                    &general_message_config_pda,
                    self.program.rpc().get_minimum_balance_for_rent_exemption(GeneralMessageConfig::MAXIMUM_SIZE)?,
                    GeneralMessageConfig::MAXIMUM_SIZE as u64,
                    &self.program.id(),
                ),
                system_program::create_account(
                    &self.payer.pubkey(),
                    &token_sender_config_pda,
                    self.program.rpc().get_minimum_balance_for_rent_exemption(SenderConfig::MAXIMUM_SIZE)?,
                    SenderConfig::MAXIMUM_SIZE as u64,
                    &self.program.id(),
                ),
                system_program::create_account(
                    &self.payer.pubkey(),
                    &redeemer_config_pda,
                    self.program.rpc().get_minimum_balance_for_rent_exemption(RedeemerConfig::MAXIMUM_SIZE)?,
                    RedeemerConfig::MAXIMUM_SIZE as u64,
                    &self.program.id(),
                ),
                system_program::create_account(
                    &self.payer.pubkey(),
                    &wormhole_emitter_pda,
                    self.program.rpc().get_minimum_balance_for_rent_exemption(WormholeEmitter::MAXIMUM_SIZE)?,
                    WormholeEmitter::MAXIMUM_SIZE as u64,
                    &self.program.id(),
                ),
        ];

        // 4. Create the initialize instruction with all required accounts
        let accounts = vec![
            // Config Accounts
            AccountMeta::new(self.payer.pubkey(), true),                    // owner/payer
            AccountMeta::new(general_message_config_pda, false),            // general message config PDA
            AccountMeta::new(token_sender_config_pda, false),               // token sender config PDA
            AccountMeta::new(redeemer_config_pda, false),                   // token redeemer config PDA

            // Wormhole Program Accounts
            AccountMeta::new_readonly(wormhole_program, false),             // wormhole program
            AccountMeta::new_readonly(wormhole_addresses.bridge, false),              // wormhole bridge
            AccountMeta::new(wormhole_addresses.fee_collector, false),                // wormhole fee collector
            AccountMeta::new(wormhole_addresses.sequence, false),                     // wormhole sequence
            AccountMeta::new(wormhole_emitter, false),                      // wormhole emitter

            // Token Bridge Program Accounts
            AccountMeta::new_readonly(token_bridge_program, false),         // token bridge program
            AccountMeta::new_readonly(outbound_token_bridge_addresses.config, false),          // outbound token bridge config
            AccountMeta::new_readonly(outbound_token_bridge_addresses.authority_signer, false),// outbound token bridge authority signer
            AccountMeta::new_readonly(outbound_token_bridge_addresses.custody_signer, false),  // outbound token bridge custody signer
            AccountMeta::new_readonly(outbound_token_bridge_addresses.emitter, false),         // outbound token bridge emitter
            AccountMeta::new_readonly(outbound_token_bridge_addresses.sequence, false),        // outbound token bridge sequence
            AccountMeta::new_readonly(outbound_token_bridge_addresses.wormhole_bridge, false),  // outbound token bridge wormhole bridge
            AccountMeta::new_readonly(outbound_token_bridge_addresses.wormhole_fee_collector, false),  // outbound token bridge wormhole fee collector

            AccountMeta::new_readonly(inbound_token_bridge_addresses.config, false),  // inbound token bridge config
            AccountMeta::new_readonly(inbound_token_bridge_addresses.custody_signer, false),  // inbound token bridge custody signer
            AccountMeta::new_readonly(inbound_token_bridge_addresses.mint_authority, false),  // inbound token bridge mint authority

            // System Accounts
            AccountMeta::new_readonly(system_program::id(), false),         // system program
            AccountMeta::new_readonly(clock::id(), false),                  // clock sysvar
            AccountMeta::new_readonly(rent::id(), false),                   // rent sysvar
                    
        ];

        // 5. Create instruction data
        let mut instruction_data = vec![];

        // Add instruction discriminator
        // instruction_data.extend_from_slice(&8u8.to_le_bytes()); // Anchor instruction discriminator

        instruction_data.extend_from_slice(&anchor_lang::InstructionData::Initialize {
            relayer_fee,
            relayer_fee_precision,
        }.try_to_vec()?);

        instruction_data.extend_from_slice(&[
            GeneralMessageConfig::MAXIMUM_SIZE as u8,
            SenderConfig::MAXIMUM_SIZE as u8,
            RedeemerConfig::MAXIMUM_SIZE as u8,
        ]);

        // Serialize all config data
        general_message_config.serialize(&mut instruction_data)?;
        token_sender_config.serialize(&mut instruction_data)?;
        token_redeemer_config.serialize(&mut instruction_data)?;

        // Add initial message
        let alive_message = CrossChainMessage::Alive {
            program_id: self.program.id(),
        }.try_to_vec()?;
        instruction_data.extend_from_slice(&alive_message);

        let initialize_ix = Instruction {
            program_id: self.program.id(),
            accounts,
            data: instruction_data,
        };

        // 6. Check and handle Wormhole fees
        let mut instructions = create_account_ixs;

        let bridge_data = self.program.rpc().get_account(&wormhole_addresses.bridge)?;
        let fee = bridge_data.data().get(/* fee offset */).unwrap_or(&0);

        if *fee > 0 {
            instructions.push(
                system_program::transfer(
                    &self.payer.pubkey(),
                    &wormhole_addresses.fee_collector,
                    *fee,
                )
            );
        }

        instructions.push(initialize_ix);

        let recent_blockhash = self.program.rpc().get_latest_blockhash()?;
        // let transaction = Transaction::new_signed_with_payer(
        //     // &instruction,
        //     &[instruction], // Note: Changed from &instruction to &[instruction]
        //     Some(&self.payer.pubkey()),
        //     &[&*self.payer],
        //     recent_blockhash,
        // );

        // 7. Create and send transaction
        let transaction = Transaction::new_signed_with_payer(
            &[
                &[instructions]
            ],
            Some(&self.payer.pubkey()),
            &[&*self.payer],
            recent_blockhash,
        );
    
        self.program.rpc().send_and_confirm_transaction(&transaction)?;

        Ok(())
    }

    // Add more methods for other instructions as needed
}

#[tokio::main]
async fn main() -> Result<()> {
    let payer = Keypair::new(); // In practice, load this from a file or secure source
    let url = "https://api.mainnet-beta.solana.com".to_string();
    let program_id = Pubkey::from_str("your_program_id_here")?;//wormhole program id
   
    let wormhole_client_config = WormholeClientConfig {
        // addresses: WormholeAddresses {
        //     bridge: Pubkey::from_str("wormhole_bridge_address")?,
        //     fee_collector: Pubkey::from_str("wormhole_fee_collector_address")?,
        //     sequence: Pubkey::from_str("wormhole_sequence_address")?,
        // },
        relayer_fee: 1000,           // Add this field
        relayer_fee_precision: 10000, // Add this field
    };
    let commitment = CommitmentConfig::confirmed();

    let client = WormholeClient::new(&url, payer, program_id, wormhole_client_config, commitment)?;

    // Initialize the Wormhole program
    client.initialize().await?;

    println!("Wormhole program initialized successfully!");

    Ok(())
}


        // // You need to provide the correct pubkeys for these accounts
        // let wormhole_program = Pubkey::from_str("wormhole_program_id_here")?;
        // let token_bridge_program = Pubkey::from_str("token_bridge_program_id_here")?;
        // let wormhole_bridge = Pubkey::from_str("wormhole_bridge_id_here")?;
        // let token_bridge_config = Pubkey::from_str("token_bridge_config_id_here")?;
        // let token_bridge_authority_signer = Pubkey::from_str("token_bridge_authority_signer_id_here")?;
        // let token_bridge_custody_signer = Pubkey::from_str("token_bridge_custody_signer_id_here")?;
        // let token_bridge_mint_authority = Pubkey::from_str("token_bridge_mint_authority_id_here")?;
        // let token_bridge_emitter = Pubkey::from_str("token_bridge_emitter_id_here")?;
        // let wormhole_fee_collector = Pubkey::from_str("wormhole_fee_collector_id_here")?;
        // let wormhole_sequence = Pubkey::from_str("wormhole_sequence_id_here")?;
        // let token_bridge_sequence = Pubkey::from_str("token_bridge_sequence_id_here")?;
        // let wormhole_message = Pubkey::from_str("wormhole_message_id_here")?;

        // let accounts = wormhole_accounts::Initialize {
        //     owner,
        //     general_message_config,
        //     sender_config,
        //     redeemer_config,
        //     wormhole_program,
        //     token_bridge_program,
        //     wormhole_bridge,
        //     token_bridge_config,
        //     token_bridge_authority_signer,
        //     token_bridge_custody_signer,
        //     token_bridge_mint_authority,
        //     token_bridge_emitter,
        //     wormhole_fee_collector,
        //     wormhole_emitter,
        //     wormhole_sequence,
        //     token_bridge_sequence,
        //     wormhole_message,
        //     clock: anchor_lang::solana_program::sysvar::clock::id(),
        //     rent: anchor_lang::solana_program::sysvar::rent::id(),
        //     system_program: system_program::id(),
        // };






// use anchor_lang::prelude::*;
// use wormhole_anchor_sdk::{wormhole, token_bridge};
// use crate::types::CCIHSResult;
// use crate::utility::error::CCIHSError;
// use crate::wormhole::GeneralMessageConfig;
// use crate::wormhole::WormholeError;
// use crate::protocols::wormhole::state::{ForeignEmitter, WormholeEmitter, Received, ForeignTokenEmitter, RedeemerConfig, SenderConfig};
// use anchor_spl::{
//     associated_token::AssociatedToken,
//     token::{Mint, Token, TokenAccount},
// };

// /// AKA `b"bridged"`.
// pub const SEED_PREFIX_BRIDGED: &[u8; 7] = b"bridged";
// /// AKA `b"tmp"`.
// pub const SEED_PREFIX_TMP: &[u8; 3] = b"tmp";

// /// AKA `b"general_message_config"`.
// pub const SEED_PREFIX_GENERAL_MESSAGE_CONFIG: &[u8; 18] = b"general_message_config";

// pub fn initialize_handler(ctx: Context<Initialize>, relayer_fee: u64, relayer_fee_precision: u32) -> Result<()> {
    
//     require!(
//         relayer_fee < relayer_fee_precision,
//         WormholeError::InvalidRelayerFee,
//     );

//     let general_message_config = &mut ctx.accounts.general_message_config;

//     // Set the owner of the config
//     general_message_config.owner = ctx.accounts.owner.key();

//     // Set Wormhole related addresses.
//     {
//         let wormhole = &mut general_message_config.wormhole;

//         // wormhole::BridgeData (Wormhole's program data).
//         wormhole.bridge = ctx.accounts.wormhole_bridge.key();

//         // wormhole::FeeCollector (lamports collector for posting
//         // messages).
//         wormhole.fee_collector = ctx.accounts.wormhole_fee_collector.key();

//         // wormhole::SequenceTracker (tracks # of messages posted by this
//         // program).
//         wormhole.sequence = ctx.accounts.wormhole_sequence.key();
//     }

//     // Set default values
//     general_message_config.batch_id = 0;
//     general_message_config.finality = wormhole::Finality::Confirmed as u8;

//     // Initialize our Wormhole emitter account. It is not required by the
//     // Wormhole program that there is an actual account associated with the
//     // emitter PDA. The emitter PDA is just a mechanism to have the program
//     // sign for the `wormhole::post_message` instruction.
//     //
//     // But for fun, we will store our emitter's bump for convenience.
//     // Initialize Wormhole emitter account
//     ctx.accounts.wormhole_emitter.bump = *ctx.bumps.get("wormhole_emitter").unwrap();

//     // Initialize program's sender config
//     let sender_config = &mut ctx.accounts.sender_config;

//     // Set the owner of the sender config (effectively the owner of the
//     // program).
//     sender_config.owner = ctx.accounts.owner.key();
//     sender_config.bump = ctx.bumps.sender_config;

//       // Set Token Bridge related addresses.
//       {
//         let token_bridge = &mut sender_config.token_bridge;
//         token_bridge.config = ctx.accounts.token_bridge_config.key();
//         token_bridge.authority_signer = ctx.accounts.token_bridge_authority_signer.key();
//         token_bridge.custody_signer = ctx.accounts.token_bridge_custody_signer.key();
//         token_bridge.emitter = ctx.accounts.token_bridge_emitter.key();
//         token_bridge.sequence = ctx.accounts.token_bridge_sequence.key();
//         token_bridge.wormhole_bridge = ctx.accounts.wormhole_bridge.key();
//         token_bridge.wormhole_fee_collector = ctx.accounts.wormhole_fee_collector.key();
//     }

//     // Initialize program's redeemer config
//     let redeemer_config = &mut ctx.accounts.redeemer_config;

//     // Set the owner of the redeemer config (effectively the owner of the
//     // program).
//     redeemer_config.owner = ctx.accounts.owner.key();
//     redeemer_config.bump = ctx.bumps.redeemer_config;
//     redeemer_config.relayer_fee = relayer_fee;
//     redeemer_config.relayer_fee_precision = relayer_fee_precision;

//     // Set Token Bridge related addresses.
//     {
//         let token_bridge = &mut redeemer_config.token_bridge;
//         token_bridge.config = ctx.accounts.token_bridge_config.key();
//         token_bridge.custody_signer = ctx.accounts.token_bridge_custody_signer.key();
//         token_bridge.mint_authority = ctx.accounts.token_bridge_mint_authority.key();
//     }

//     // Post initial Wormhole message
//     let fee = ctx.accounts.wormhole_bridge.fee();
//     if fee > 0 {
//         anchor_lang::solana_program::program::invoke(
//             &anchor_lang::solana_program::system_instruction::transfer(
//                 &ctx.accounts.owner.key(),
//                 &ctx.accounts.wormhole_fee_collector.key(),
//                 fee,
//             ),
//             &ctx.accounts.to_account_infos(),
//         )?;
//     }

//     let wormhole_emitter = &ctx.accounts.wormhole_emitter;

//     let payload = CrossChainMessage::Alive {
//         program_id: *ctx.program_id,
//     }.try_to_vec()?;

//     wormhole::post_message(
//         CpiContext::new_with_signer(
//             ctx.accounts.wormhole_program.to_account_info(),
//             wormhole::PostMessage {
//                 config: ctx.accounts.wormhole_bridge.to_account_info(),
//                 message: ctx.accounts.wormhole_message.to_account_info(),
//                 emitter: wormhole_emitter.to_account_info(),
//                 sequence: ctx.accounts.wormhole_sequence.to_account_info(),
//                 payer: ctx.accounts.owner.to_account_info(),
//                 fee_collector: ctx.accounts.wormhole_fee_collector.to_account_info(),
//                 clock: ctx.accounts.clock.to_account_info(),
//                 rent: ctx.accounts.rent.to_account_info(),
//                 system_program: ctx.accounts.system_program.to_account_info(),
//             },
//             &[
//                 &[
//                     SEED_PREFIX_SENT,
//                     &wormhole::INITIAL_SEQUENCE.to_le_bytes()[..],
//                     &[*ctx.bumps.get("wormhole_message").unwrap()],
//                 ],
//                 &[wormhole::SEED_PREFIX_EMITTER, &[wormhole_emitter.bump]],
//             ],
//         ),
//         general_message_config.batch_id,
//         payload,
//         general_message_config.finality.try_into().unwrap(),
//     )?;

//     Ok(())
// }

// #[derive(Accounts)]
// /// Context used to initialize program data (i.e. config).
// pub struct Initialize<'info> {
//     #[account(mut)]
//     /// Whoever initializes the configs will be the owner of the program. Signer
//     /// for creating the [`Config`] accounts and posting a Wormhole message and token transfers
//     /// indicating that the program is alive.
//     pub owner: Signer<'info>,

//     #[account(
//         init,
//         payer = owner,
//         seeds = [GeneralMessageConfig::SEED_PREFIX],
//         bump,
//         space = GeneralMessageConfig::MAXIMUM_SIZE,
//     )]
//     /// General message config account, which saves program data useful for other instructions.
//     /// Also saves the payer of the [`initialize`](crate::initialize) instruction
//     /// as the program's owner.
//     pub general_message_config: Account<'info, GeneralMessageConfig>,

//     #[account(
//         init,
//         payer = owner,
//         seeds = [SenderConfig::SEED_PREFIX],
//         bump,
//         space = SenderConfig::MAXIMUM_SIZE,
//     )]
//     /// Sender Config account, which saves program data useful for other
//     /// instructions, specifically for outbound transfers. Also saves the payer
//     /// of the [`initialize`](crate::initialize) instruction as the program's
//     /// owner.
//     pub sender_config: Box<Account<'info, SenderConfig>>,

//     #[account(
//         init,
//         payer = owner,
//         seeds = [RedeemerConfig::SEED_PREFIX],
//         bump,
//         space = RedeemerConfig::MAXIMUM_SIZE,
//     )]
//     /// Redeemer Config account, which saves program data useful for other
//     /// instructions, specifically for inbound transfers. Also saves the payer
//     /// of the [`initialize`](crate::initialize) instruction as the program's
//     /// owner.
//     pub redeemer_config: Box<Account<'info, RedeemerConfig>>,

//     /// Wormhole program.
//     pub wormhole_program: Program<'info, wormhole::program::Wormhole>,

//     /// Token Bridge program.
//     pub token_bridge_program: Program<'info, token_bridge::program::TokenBridge>,

//     // #[account(
//     //     mut,
//     //     seeds = [wormhole::BridgeData::SEED_PREFIX],
//     //     bump,
//     //     seeds::program = wormhole_program,
//     // )]
//     // /// Wormhole bridge data account (a.k.a. its config).
//     // /// [`wormhole::post_message`] requires this account be mutable.
//     // pub wormhole_bridge: Account<'info, wormhole::BridgeData>,

//     #[account(
//         seeds = [wormhole::BridgeData::SEED_PREFIX],
//         bump,
//         seeds::program = wormhole_program,
//     )]
//     /// Wormhole bridge data account (a.k.a. its config).
//     pub wormhole_bridge: Box<Account<'info, wormhole::BridgeData>>,

//     #[account(
//         seeds = [token_bridge::Config::SEED_PREFIX],
//         bump,
//         seeds::program = token_bridge_program,
//     )]
//     /// Token Bridge config. Token Bridge program needs this account to
//     /// invoke the Wormhole program to post messages. Even though it is a
//     /// required account for redeeming token transfers, it is not actually
//     /// used for completing these transfers.
//     pub token_bridge_config: Account<'info, token_bridge::Config>,

//     #[account(
//         seeds = [token_bridge::SEED_PREFIX_AUTHORITY_SIGNER],
//         bump,
//         seeds::program = token_bridge_program,
//     )]
//     /// CHECK: Token Bridge authority signer. This isn't an account that holds
//     /// data; it is purely just a signer for SPL tranfers when it is delegated
//     /// spending approval for the SPL token.
//     pub token_bridge_authority_signer: UncheckedAccount<'info>,

//     #[account(
//         seeds = [token_bridge::SEED_PREFIX_CUSTODY_SIGNER],
//         bump,
//         seeds::program = token_bridge_program,
//     )]
//     /// CHECK: Token Bridge custody signer. This isn't an account that holds
//     /// data; it is purely just a signer for Token Bridge SPL tranfers.
//     pub token_bridge_custody_signer: UncheckedAccount<'info>,


//     #[account(
//         seeds = [token_bridge::SEED_PREFIX_MINT_AUTHORITY],
//         bump,
//         seeds::program = token_bridge_program,
//     )]
//     /// CHECK: Token Bridge mint authority. This isn't an account that holds
//     /// data; it is purely just a signer (SPL mint authority) for Token Bridge
//     /// wrapped assets.
//     pub token_bridge_mint_authority: UncheckedAccount<'info>,

//     #[account(
//         seeds = [token_bridge::SEED_PREFIX_EMITTER],
//         bump,
//         seeds::program = token_bridge_program
//     )]
//     /// CHECK: Token Bridge program's emitter account. This isn't an account
//     /// that holds data; it is purely just a signer for posting Wormhole
//     /// messages on behalf of the Token Bridge program.
//     pub token_bridge_emitter: UncheckedAccount<'info>,

//     #[account(
//         mut,
//         seeds = [wormhole::FeeCollector::SEED_PREFIX],
//         bump,
//         seeds::program = wormhole_program
//     )]
//     /// Wormhole fee collector account, which requires lamports before the
//     /// program can post a message (if there is a fee).
//     /// [`wormhole::post_message`] requires this account be mutable.
//     pub wormhole_fee_collector: Account<'info, wormhole::FeeCollector>,

//     #[account(
//         init,
//         payer = owner,
//         seeds = [WormholeEmitter::SEED_PREFIX],
//         bump,
//         space = WormholeEmitter::MAXIMUM_SIZE
//     )]
//     /// This program's emitter account. We create this account in the
//     /// [`initialize`](crate::initialize) instruction, but
//     /// [`wormhole::post_message`] only needs it to be read-only.
//     pub wormhole_emitter: Account<'info, WormholeEmitter>,

//     #[account(
//         mut,
//         seeds = [
//             wormhole::SequenceTracker::SEED_PREFIX,
//             wormhole_emitter.key().as_ref()
//         ],
//         bump,
//         seeds::program = wormhole_program
//     )]
//     /// CHECK: Emitter's sequence account. This is not created until the first
//     /// message is posted, so it needs to be an [UncheckedAccount] for the
//     /// [`initialize`](crate::initialize) instruction.
//     /// [`wormhole::post_message`] requires this account be mutable.
//     pub wormhole_sequence: UncheckedAccount<'info>,

//     #[account(
//         seeds = [
//             wormhole::SequenceTracker::SEED_PREFIX,
//             token_bridge_emitter.key().as_ref()
//         ],
//         bump,
//         seeds::program = wormhole_program
//     )]
//     /// Token Bridge emitter's sequence account. Like with all Wormhole
//     /// emitters, this account keeps track of the sequence number of the last
//     /// posted message.
//     pub token_bridge_sequence: Account<'info, wormhole::SequenceTracker>,

//     #[account(
//         mut,
//         seeds = [
//             SEED_PREFIX_SENT,
//             &wormhole::INITIAL_SEQUENCE.to_le_bytes()[..]
//         ],
//         bump,
//     )]
//     /// CHECK: Wormhole message account. The Wormhole program writes to this
//     /// account, which requires this program's signature.
//     /// [`wormhole::post_message`] requires this account be mutable.
//     pub wormhole_message: UncheckedAccount<'info>,

//     /// Clock sysvar.
//     pub clock: Sysvar<'info, Clock>,

//     /// Rent sysvar.
//     pub rent: Sysvar<'info, Rent>,

//     /// System program.
//     pub system_program: Program<'info, System>,
// }