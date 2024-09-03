use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke_signed;
use crate::protocols::ProtocolAdapter;
use std::io;
use wormhole_io::{Readable, Writeable};
use super::state::*;
use std::collections::BTreeMap;


use wormhole_anchor_sdk::{wormhole, token_bridge};
use wormhole_anchor_sdk::token_bridge::{
    program::TokenBridge,
    cpi::accounts::{CompleteNative, CompleteTransferNativeWithPayload, TransferNative},
    ConfigAccount as TokenBridgeConfig,
};
use wormhole_anchor_sdk::wormhole::{
    program::Wormhole,
    state::{PostedVaa, BridgeData},
    VerifySignatures,
};

use crate::types::{ ChainId, CrossChainAddress, CCIHSResult, MessageStatus, HookType};
use crate::protocols::wormhole::message::*;
use crate::error::CCIHSError;
use crate::hooks::HookManager;
use super::config::WormholeConfig;
use super::instructions::*;

pub struct WormholeAdapter {
    pub config: WormholeConfig, //I still need to setup WormholeConfig
    hook_manager: HookManager,
    foreign_emitters: ForeignEmitter,
    received: Received,
}

impl WormholeAdapter {
    pub fn new(config: WormholeConfig, hook_manager: HookManager) -> Self {
        Self { config, hook_manager, foreign_emitters: BTreeMap::new(), received: Received::default() }
    }

    pub fn initialize(&self, ctx: Context<Initialize>) -> Result<()> {
        initialize_handler(ctx, self.config.relayer_fee, self.config.relayer_fee_precision)//TODO: Check to know if you need to add the args to WormholeConfig
    }

    pub fn register_emitter(&self, ctx: Context<RegisterEmitter>, chain: u16, address: [u8; 32]) -> Result<()> {
        register_emitter_handler(ctx, chain, address)//TODO: Check to know if you need to add the args to WormholeConfig
    }

    pub fn send_message(&self, ctx: Context<SendMessage>, message: &CrossChainMessage) -> Result<()> {
        send_message_handler(ctx, message)//TODO: Check to know if you need to add the args to WormholeConfig
    }

    fn serialize_message(&self, message: &CrossChainMessage) -> Result<Vec<u8>> {
        wormhole_io::serialize(message)
        .map_err(|e| CCIHSError::SerializationError(e.to_string()))
    }

    fn deserialize_message(&self, payload: &[u8]) -> Result<CrossChainMessage> {
        wormhole_io::deserialize(payload)
        .map_err(|e| CCIHSError::DeserializationError(e.to_string()))    
    }
    pub fn add_foreign_emitter(&mut self, chain: u16, address: [u8; 32]) {
        self.foreign_emitters.insert(chain, ForeignEmitter { chain, address });
    }

    pub fn get_foreign_emitter(&self, chain: u16) -> Option<&ForeignEmitter> {
        self.foreign_emitters.get(&chain)
    }

    pub fn remove_foreign_emitter(&mut self, chain: u16) -> Option<ForeignEmitter> {
        self.foreign_emitters.remove(&chain)
    }

    pub fn verify_foreign_emitter(&self, chain: u16, address: &[u8; 32]) -> bool {
        self.foreign_emitters
            .get(&chain)
            .map_or(false, |emitter| emitter.verify(address))
    }

    pub fn update_received(&mut self, batch_id: u32) {
        self.received.batch_id = batch_id;
        self.received.message_count += 1;
    }

    pub fn get_received(&self) -> &Received {
        &self.received
    }

    // fn get_emitter_address(&self, program_id: &Pubkey) -> CCIHSResult<Pubkey> {
    //     let seeds = [b"emitter"];
    //     let (emitter_address, _) = Pubkey::find_program_address(&seeds, program_id);
    //     Ok(emitter_address)
    // }

    // fn get_sequence_address(&self, program_id: &Pubkey, emitter_address: &Pubkey) -> CCIHSResult<Pubkey> {
    //     let seeds = [b"sequence", emitter_address.as_ref()];
    //     let (sequence_address, _) = Pubkey::find_program_address(&seeds, program_id);
    //     Ok(sequence_address)
    // }

    // pub fn update_config(&mut self, new_config: WormholeConfig) -> CCIHSResult<()> {
    //     // Perform any necessary validation
    //     if new_config.fee < self.config.fee {
    //         return Err(CCIHSError::InvalidConfiguration("Fee cannot be decreased".to_string()));
    //     }
    //     self.config = new_config;
    //     Ok(())
    // }

    // pub fn get_current_sequence(&self, program_id: &Pubkey) -> CCIHSResult<u64> {
    //     let emitter_address = self.get_emitter_address(program_id)?;
    //     let sequence_address = self.get_sequence_address(program_id, &emitter_address)?;
        
    //     // Fetch the account data for the sequence address
    //     // This is a placeholder and should be replaced with actual Solana account data fetching
    //     let sequence_data = vec![0u8; 8]; // Placeholder
        
    //     let sequence = u64::from_le_bytes(sequence_data.try_into().map_err(|_| CCIHSError::InvalidSequenceData)?);
    //     Ok(sequence)
    // }

    fn verify_consistency_level(&self, vaa: &PostedVaa) -> CCIHSResult<()> {
        if vaa.consistency_level() < self.config.consistency_level {
            return Err(CCIHSError::InvalidConsistencyLevel);
        }
        Ok(())
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
        
        // Check if the emitter is known and verified
        let emitter_chain = posted_vaa.emitter_chain();
        let emitter_address = posted_vaa.emitter_address();
        if !self.verify_foreign_emitter(emitter_chain, &emitter_address) {
            return Err(CCIHSError::UnknownEmitter.into());
        }

        let message = self.deserialize_message(&posted_vaa.payload)?;

        // Update received messages
        self.update_received(posted_vaa.batch_id());

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
