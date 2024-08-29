use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke_signed;
use crate::protocols::ProtocolAdapter;
use std::io;
use wormhole_io::{Readable, Writeable};

use wormhole_anchor_sdk::{wormhole, token_bridge};
use wormhole_anchor_sdk::token_bridge::{
    program::TokenBridge,
    cpi::accounts::{CompleteNative, TransferNative},
    ConfigAccount as TokenBridgeConfig,
};
use wormhole_anchor_sdk::wormhole::{
    program::Wormhole,
    state::{PostedVaa, BridgeData},
    VerifySignatures,
};

use crate::types::{CrossChainMessage, ChainId, CrossChainAddress, CCIHSResult, MessageStatus, HookType};
use crate::error::CCIHSError;
use crate::hooks::HookManager;
use super::config::WormholeConfig;

pub struct WormholeAdapter {
    pub config: WormholeConfig,
    hook_manager: HookManager,
}

impl WormholeAdapter {
    pub fn new(config: WormholeConfig, hook_manager: HookManager) -> Self {
        Self { config, hook_manager }
    }

    fn serialize_message(&self, message: &CrossChainMessage) -> Result<Vec<u8>> {
        wormhole_io::serialize(message)
        .map_err(|e| CCIHSError::SerializationError(e.to_string()))
    }

    fn deserialize_message(&self, payload: &[u8]) -> Result<CrossChainMessage> {
        wormhole_io::deserialize(payload)
        .map_err(|e| CCIHSError::DeserializationError(e.to_string()))    
    }
}

impl ProtocolAdapter for WormholeAdapter {
    fn send_message<'info>(
        &self,
        ctx: Context<'_, '_, '_, 'info, SendMessage<'info>>,
        message: &CrossChainMessage,
    ) -> Result<()> {
        // Execute pre-dispatch hooks
        self.hook_manager.execute_hooks(HookType::PreDispatch, &mut message, message.source_chain, message.destination_chain)?;

        let payload = self.serialize_message(message)?;
        
        // Post the message
        wormhole::post_message(
            CpiContext::new(
                ctx.accounts.wormhole_program.to_account_info(),
                wormhole::PostMessage {
                    config: ctx.accounts.wormhole_config.to_account_info(),
                    message: ctx.accounts.wormhole_message.to_account_info(),
                    emitter: ctx.accounts.wormhole_emitter.to_account_info(),
                    sequence: ctx.accounts.wormhole_sequence.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    fee_collector: ctx.accounts.wormhole_fee_collector.to_account_info(),
                    clock: ctx.accounts.clock.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                },
            ),
            config.batch_id,
            payload,
            config.finality.into(),
        )?;

        // Execute token bridge transfer
        token_bridge::transfer_native(
            CpiContext::new(
                ctx.accounts.token_bridge_program.to_account_info(),
                TransferNative {
                    payer: ctx.accounts.payer.to_account_info(),
                    config: ctx.accounts.token_bridge_config.to_account_info(),
                    from: ctx.accounts.from_token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    custody: ctx.accounts.token_bridge_custody.to_account_info(),
                    authority_signer: ctx.accounts.token_bridge_authority_signer.to_account_info(),
                    custody_signer: ctx.accounts.token_bridge_custody_signer.to_account_info(),
                    wormhole_bridge: ctx.accounts.wormhole_config.to_account_info(),
                    wormhole_message: ctx.accounts.wormhole_message.to_account_info(),
                    wormhole_emitter: ctx.accounts.wormhole_emitter.to_account_info(),
                    wormhole_sequence: ctx.accounts.wormhole_sequence.to_account_info(),
                    wormhole_fee_collector: ctx.accounts.wormhole_fee_collector.to_account_info(),
                    clock: ctx.accounts.clock.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    wormhole_program: ctx.accounts.wormhole_program.to_account_info(),
                },
            ),
            message.amount,
            message.recipient.to_vec(),
            message.destination_chain.0,
            message.nonce,
            message.consistency_level,
        )?;

        // Execute post-dispatch hooks
        self.hook_manager.execute_hooks(HookType::PostDispatch, &mut message, message.source_chain, message.destination_chain)?;

        Ok(())
    }

    fn receive_message<'info>(
        &self,
        ctx: Context<'_, '_, '_, 'info, ReceiveMessage<'info>>,
    ) -> Result<CrossChainMessage> {
        // Verify and post the VAA
        wormhole::verify_signature(
            CpiContext::new(
                ctx.accounts.wormhole_program.to_account_info(),
                VerifySignatures {
                    payer: ctx.accounts.payer.to_account_info(),
                    guardian_set: ctx.accounts.guardian_set.to_account_info(),
                    signature_set: ctx.accounts.signature_set.to_account_info(),
                    instructions: ctx.accounts.instructions.to_account_info(),
                },
            ),
            ctx.accounts.vaa.to_account_info(),
        )?;

        wormhole::post_vaa(
            CpiContext::new(
                ctx.accounts.wormhole_program.to_account_info(),
                wormhole::PostVaa {
                    payer: ctx.accounts.payer.to_account_info(),
                    signature_set: ctx.accounts.signature_set.to_account_info(),
                    post_vaa: ctx.accounts.posted_vaa.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                },
            ),
            ctx.accounts.vaa.to_account_info(),
        )?;

        // Deserialize the VAA
        let posted_vaa = PostedVaa::try_from_slice(&ctx.accounts.posted_vaa.data.borrow())?;
        let message = self.deserialize_message(&posted_vaa.payload)?;

        // Execute pre-execution hooks
        self.hook_manager.execute_hooks(HookType::PreExecution, &mut message, message.source_chain, message.destination_chain)?;

        // Complete token transfer
        token_bridge::complete_native(
            CpiContext::new(
                ctx.accounts.token_bridge_program.to_account_info(),
                CompleteNative {
                    payer: ctx.accounts.payer.to_account_info(),
                    config: ctx.accounts.token_bridge_config.to_account_info(),
                    vaa: ctx.accounts.posted_vaa.to_account_info(),
                    claim: ctx.accounts.claim.to_account_info(),
                    foreign_endpoint: ctx.accounts.foreign_endpoint.to_account_info(),
                    to: ctx.accounts.to_token_account.to_account_info(),
                    redeemer: ctx.accounts.redeemer.to_account_info(),
                    custody: ctx.accounts.custody_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    custody_signer: ctx.accounts.custody_signer.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    wormhole_program: ctx.accounts.wormhole_program.to_account_info(),
                },
            ),
        )?;

        // Execute post-execution hooks
        self.hook_manager.execute_hooks(HookType::PostExecution, &mut message, message.source_chain, message.destination_chain)?;

        Ok(message)
    }

    fn verify_message<'info>(
        &self,
        ctx: Context<'_, '_, '_, 'info, VerifyMessage<'info>>,
    ) -> Result<bool> {
        wormhole::verify_signature(
            CpiContext::new(
                ctx.accounts.wormhole_program.to_account_info(),
                wormhole::VerifySignature {
                    payer: ctx.accounts.payer.to_account_info(),
                    guardian_set: ctx.accounts.guardian_set.to_account_info(),
                    signature_set: ctx.accounts.signature_set.to_account_info(),
                    instructions: ctx.accounts.instructions.to_account_info(),
                },
            ),
            ctx.accounts.vaa.to_account_info(),
        )?;

        Ok(true)
    }
    fn supported_chains(&self) -> Vec<ChainId> {
        self.config.supported_chains()
    }
}

#[derive(Accounts)]
pub struct SendMessage<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub from_token_account: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub wormhole_message: AccountInfo<'info>,
    pub wormhole_config: Account<'info, BridgeData>,
    pub token_bridge_config: Account<'info, TokenBridgeConfig>,
    #[account(mut)]
    pub token_bridge_custody: AccountInfo<'info>,
    pub token_bridge_authority_signer: AccountInfo<'info>,
    pub token_bridge_custody_signer: AccountInfo<'info>,
    pub wormhole_emitter: AccountInfo<'info>,
    #[account(mut)]
    pub wormhole_sequence: AccountInfo<'info>,
    #[account(mut)]
    pub wormhole_fee_collector: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub wormhole_program: Program<'info, Wormhole>,
    pub token_bridge_program: Program<'info, TokenBridge>,
}

#[derive(Accounts)]
pub struct ReceiveMessage<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub vaa: AccountInfo<'info>,
    #[account(mut)]
    pub posted_vaa: AccountInfo<'info>,
    #[account(mut)]
    pub signature_set: AccountInfo<'info>,
    pub guardian_set: AccountInfo<'info>,
    pub token_bridge_config: Account<'info, TokenBridgeConfig>,
    #[account(mut)]
    pub to_token_account: Account<'info, TokenAccount>,
    pub redeemer: AccountInfo<'info>,
    pub foreign_endpoint: AccountInfo<'info>,
    #[account(mut)]
    pub claim: AccountInfo<'info>,
    pub custody_signer: AccountInfo<'info>,
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub custody_account: Account<'info, TokenAccount>,
    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub wormhole_program: Program<'info, Wormhole>,
    pub token_bridge_program: Program<'info, TokenBridge>,
    pub instructions: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct VerifyMessage<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub guardian_set: AccountInfo<'info>,
    pub vaa: AccountInfo<'info>,
    #[account(mut)]
    pub signature_set: AccountInfo<'info>,
    pub instruction: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub wormhole_program: Program<'info, Wormhole>,
}


//To use in your program

// use anchor_lang::prelude::*;
// use your_crate::{
//     protocol::{ProtocolAdapter, WormholeAdapter, WormholeConfig},
//     types::{CrossChainMessage, ChainId},
// };

// declare_id!("Your_Program_ID");

// #[program]
// pub mod your_program {
//     use super::*;

//     pub fn initialize(ctx: Context<Initialize>, wormhole_config: WormholeConfig) -> Result<()> {
//         let adapter = WormholeAdapter::new(wormhole_config);
//         ctx.accounts.protocol_state.adapter = adapter;
//         Ok(())
//     }

//     pub fn send_cross_chain_message(ctx: Context<SendMessage>, message: CrossChainMessage) -> Result<()> {
//         ctx.accounts.protocol_state.adapter.send_message(
//             &message,
//             &ctx.accounts.payer,
//             &ctx.accounts.wormhole_message,
//             &ctx.accounts.wormhole_bridge_config,
//             &ctx.accounts.fee_collector,
//             &ctx.accounts.sequence,
//             &ctx.accounts.clock,
//             &ctx.accounts.rent,
//             &ctx.accounts.system_program,
//         )
//     }

//     pub fn receive_cross_chain_message(ctx: Context<ReceiveMessage>) -> Result<()> {
//         let message = ctx.accounts.protocol_state.adapter.receive_message(&ctx.accounts.vaa_account)?;
//         // Process the received message
//         msg!("Received message from chain: {:?}", message.source_chain);
//         Ok(())
//     }

//     pub fn verify_cross_chain_message(ctx: Context<VerifyMessage>) -> Result<()> {
//         let is_valid = ctx.accounts.protocol_state.adapter.verify_message(&ctx.accounts.vaa_account)?;
//         if is_valid {
//             msg!("Message verified successfully");
//         } else {
//             msg!("Message verification failed");
//         }
//         Ok(())
//     }
// }

// #[derive(Accounts)]
// pub struct Initialize<'info> {
//     #[account(init, payer = payer, space = 8 + std::mem::size_of::<ProtocolState>())]
//     pub protocol_state: Account<'info, ProtocolState>,
//     #[account(mut)]
//     pub payer: Signer<'info>,
//     pub system_program: Program<'info, System>,
// }

// #[derive(Accounts)]
// pub struct SendMessage<'info> {
//     #[account(mut)]
//     pub protocol_state: Account<'info, ProtocolState>,
//     #[account(mut)]
//     pub payer: Signer<'info>,
//     /// CHECK: This is the Wormhole message account
//     #[account(mut)]
//     pub wormhole_message: AccountInfo<'info>,
//     /// CHECK: This is the Wormhole bridge config
//     pub wormhole_bridge_config: AccountInfo<'info>,
//     /// CHECK: This is the fee collector account
//     pub fee_collector: AccountInfo<'info>,
//     /// CHECK: This is the sequence account
//     pub sequence: AccountInfo<'info>,
//     pub clock: Sysvar<'info, Clock>,
//     pub rent: Sysvar<'info, Rent>,
//     pub system_program: Program<'info, System>,
// }

// #[derive(Accounts)]
// pub struct ReceiveMessage<'info> {
//     #[account(mut)]
//     pub protocol_state: Account<'info, ProtocolState>,
//     /// CHECK: This is the VAA account
//     pub vaa_account: AccountInfo<'info>,
// }

// #[derive(Accounts)]
// pub struct VerifyMessage<'info> {
//     pub protocol_state: Account<'info, ProtocolState>,
//     /// CHECK: This is the VAA account
//     pub vaa_account: AccountInfo<'info>,
// }

// #[account]
// pub struct ProtocolState {
//     pub adapter: WormholeAdapter,
// }




// use super::ProtocolAdapter;
// use crate::types::{CrossChainMessage, ChainId, CCIHSResult};
// use solana_program::pubkey::Pubkey;

// use anchor_lang::prelude::*;
// use wormhole_sdk::{
//     token_bridge,
//     vaa::{VAA, DeserializePayload},
// };

// use wormhole_token_bridge_solana::{sdk, instruction};
// use crate::types::{CrossChainMessage, ChainId};
// use crate::error::CCIHSError;

// #[derive(AnchorSerialize, AnchorDeserialize, Clone)]
// pub struct WormholeConfig {
//     pub program_id: Pubkey,
//     pub token_bridge_program_id: Pubkey,
//     pub wormhole_bridge_config: Pubkey,
//     pub token_bridge_config: Pubkey,
//     pub fee_collector: Pubkey,
//     pub sequence: Pubkey,
//     pub emitter: Pubkey,
// }

// #[derive(AnchorSerialize, AnchorDeserialize)]
// pub struct WormholeAdapter {
//     pub config: WormholeConfig,
// }

// impl WormholeAdapter {
//     pub fn new(config: WormholeConfig) -> Self {
//         Self { config }
//     }

//     pub fn send_message<'info>(
//         &self,
//         message: &CrossChainMessage,
//         payer: &Signer<'info>,
//         wormhole_message: &AccountInfo<'info>,
//         wormhole_bridge_config: &AccountInfo<'info>,
//         fee_collector: &AccountInfo<'info>,
//         sequence: &AccountInfo<'info>,
//         clock: &Sysvar<'info, Clock>,
//         rent: &Sysvar<'info, Rent>,
//         system_program: &Program<'info, System>,
//     ) -> Result<()> {
//         let payload = self.serialize_message(message)?;
        
//         // Create Wormhole message account
//         let space = 1000; // Adjust based on actual requirements
//         let lamports = rent.minimum_balance(space);
        
//         anchor_lang::system_program::create_account(
//             CpiContext::new(
//                 system_program.to_account_info(),
//                 anchor_lang::system_program::CreateAccount {
//                     from: payer.to_account_info(),
//                     to: wormhole_message.to_account_info(),
//                 },
//             ),
//             lamports,
//             space as u64,
//             &self.config.program_id,
//         )?;

//         // Post message to Wormhole
//         let instruction = token_bridge::instructions::post_message(
//             &self.config.program_id,
//             &self.config.token_bridge_program_id,
//             &self.config.wormhole_bridge_config,
//             &self.config.token_bridge_config,
//             &self.config.fee_collector,
//             &self.config.sequence,
//             &self.config.emitter,
//             &payload,
//         ).map_err(|e| error!(CCIHSError::WormholeError(e.to_string())))?;

//         anchor_lang::solana_program::program::invoke(
//             &instruction,
//             &[
//                 payer.to_account_info(),
//                 wormhole_message.clone(),
//                 wormhole_bridge_config.clone(),
//                 fee_collector.clone(),
//                 sequence.clone(),
//                 clock.to_account_info(),
//                 rent.to_account_info(),
//             ],
//         ).map_err(|e| error!(CCIHSError::InstructionExecutionFailed(e.to_string())))?;

//         Ok(())
//     }

//     pub fn receive_message(&self, vaa_account: &AccountInfo) -> Result<CrossChainMessage> {
//         let vaa_data = vaa_account.try_borrow_data()?;
//         let vaa = self.parse_vaa(&vaa_data)?;
//         self.deserialize_message(&vaa.payload)
//     }

//     pub fn verify_message(&self, vaa_account: &AccountInfo) -> Result<bool> {
//         let vaa_data = vaa_account.try_borrow_data()?;
//         let vaa = self.parse_vaa(&vaa_data)?;
        
//         // In a real implementation, you'd verify the VAA against the current guardian set
//         // This is a placeholder check
//         Ok(!vaa.signatures.is_empty())
//     }

//     fn serialize_message(&self, message: &CrossChainMessage) -> Result<Vec<u8>> {
//          // Implement serialization logic
//         // This is a placeholder; adjust according to your CrossChainMessage structure
//         message.try_to_vec().map_err(|e| error!(CCIHSError::SerializationError))
//     }

//     fn deserialize_message(&self, data: &[u8]) -> Result<CrossChainMessage> {
//         // Implement deserialization logic
//         // This is a placeholder; adjust according to your CrossChainMessage structure
//         CrossChainMessage::try_from_slice(data).map_err(|e| error!(CCIHSError::DeserializationError))
//     }

//     fn parse_vaa(&self, vaa_bytes: &[u8]) -> Result<VAA<DeserializePayload>> {
//         VAA::deserialize(vaa_bytes)
//             .map_err(|e| error!(CCIHSError::WormholeError(format!("Failed to parse VAA: {}", e))))
//     }
// }

//use this in your program
// use anchor_lang::prelude::*;
// use your_crate::{WormholeAdapter, WormholeConfig};

// #[program]
// pub mod your_program {
//     use super::*;

//     pub fn send_cross_chain_message(ctx: Context<SendMessage>, message: CrossChainMessage) -> Result<()> {
//         let wormhole_config = WormholeConfig {
//             program_id: ctx.accounts.wormhole_program.key(),
//             token_bridge_program_id: ctx.accounts.token_bridge_program.key(),
//             wormhole_bridge_config: ctx.accounts.wormhole_bridge_config.key(),
//             token_bridge_config: ctx.accounts.token_bridge_config.key(),
//             fee_collector: ctx.accounts.fee_collector.key(),
//             sequence: ctx.accounts.sequence.key(),
//             emitter: ctx.accounts.emitter.key(),
//         };
//         let adapter = WormholeAdapter::new(wormhole_config);
//         adapter.send_message(
//             &message,
//             &ctx.accounts.payer,
//             &ctx.accounts.wormhole_message,
//             &ctx.accounts.wormhole_bridge_config,
//             &ctx.accounts.fee_collector,
//             &ctx.accounts.sequence,
//             &ctx.accounts.clock,
//             &ctx.accounts.rent,
//             &ctx.accounts.system_program,
//         )
//     }
// }

// #[derive(Accounts)]
// pub struct SendMessage<'info> {
//     #[account(mut)]
//     pub payer: Signer<'info>,
//     #[account(mut)]
//     pub wormhole_message: AccountInfo<'info>,
//     pub wormhole_program: AccountInfo<'info>,
//     pub token_bridge_program: AccountInfo<'info>,
//     pub wormhole_bridge_config: AccountInfo<'info>,
//     pub token_bridge_config: AccountInfo<'info>,
//     pub fee_collector: AccountInfo<'info>,
//     pub sequence: AccountInfo<'info>,
//     pub emitter: AccountInfo<'info>,
//     pub clock: Sysvar<'info, Clock>,
//     pub rent: Sysvar<'info, Rent>,
//     pub system_program: Program<'info, System>,
// }