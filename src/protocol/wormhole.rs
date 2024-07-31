// src/protocol/wormhole.rs

use super::ProtocolAdapter;
use anchor_lang::prelude::*;
use wormhole_token_bridge_solana::sdk::{
    post_message,
    verify_signature,
    parse_vaa,
    VAA,
};
use crate::types::{CrossChainMessage, ChainId, CrossChainAddress};
use crate::error::CCIHSError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct WormholeConfig {
    pub wormhole_program: Pubkey,
    pub token_bridge_program: Pubkey,
    pub wormhole_bridge_config: Pubkey,
    pub token_bridge_config: Pubkey,
    pub fee_collector: Pubkey,
    pub sequence: Pubkey,
    pub emitter: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WormholeAdapter {
    pub config: WormholeConfig,
}

impl WormholeAdapter {
    pub fn new(config: WormholeConfig) -> Self {
        Self { config }
    }

    fn serialize_message(&self, message: &CrossChainMessage) -> Result<Vec<u8>> {
        let mut serialized = Vec::new();
        serialized.extend_from_slice(&message.source_chain.0.to_le_bytes());
        serialized.extend_from_slice(&message.destination_chain.0.to_le_bytes());
        serialized.extend_from_slice(message.sender.as_ref());
        serialized.extend_from_slice(&(message.recipient.len() as u16).to_le_bytes());
        serialized.extend_from_slice(&message.recipient);
        serialized.extend_from_slice(&(message.payload.len() as u32).to_le_bytes());
        serialized.extend_from_slice(&message.payload);
        serialized.extend_from_slice(&message.nonce.to_le_bytes());
        serialized.extend_from_slice(&message.timestamp.to_le_bytes());
        Ok(serialized)
    }

    fn deserialize_message(&self, data: &[u8]) -> Result<CrossChainMessage> {
        let mut cursor = std::io::Cursor::new(data);
        use byteorder::{LittleEndian, ReadBytesExt};

        let source_chain = ChainId(cursor.read_u16::<LittleEndian>()?);
        let destination_chain = ChainId(cursor.read_u16::<LittleEndian>()?);
        let mut sender = [0u8; 32];
        cursor.read_exact(&mut sender)?;
        let recipient_len = cursor.read_u16::<LittleEndian>()? as usize;
        let mut recipient = vec![0u8; recipient_len];
        cursor.read_exact(&mut recipient)?;
        let payload_len = cursor.read_u32::<LittleEndian>()? as usize;
        let mut payload = vec![0u8; payload_len];
        cursor.read_exact(&mut payload)?;
        let nonce = cursor.read_u64::<LittleEndian>()?;
        let timestamp = cursor.read_i64::<LittleEndian>()?;

        Ok(CrossChainMessage {
            source_chain,
            destination_chain,
            sender: Pubkey::new_from_array(sender),
            recipient,
            payload,
            nonce,
            timestamp,
        })
    }
}

impl ProtocolAdapter for WormholeAdapter {
    fn send_message<'info>(
        &self,
        message: &CrossChainMessage,
        payer: &Signer<'info>,
        wormhole_message: &AccountInfo<'info>,
        wormhole_bridge_config: &AccountInfo<'info>,
        fee_collector: &AccountInfo<'info>,
        sequence: &AccountInfo<'info>,
        clock: &Sysvar<'info, Clock>,
        rent: &Sysvar<'info, Rent>,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        let payload = self.serialize_message(message)?;
        
        post_message(
            self.config.wormhole_program,
            self.config.token_bridge_program,
            *payer.key,
            self.config.wormhole_bridge_config,
            *wormhole_message.key,
            self.config.emitter,
            self.config.sequence,
            payload,
            message.nonce,
            1, // consistency_level, adjust as needed
        ).map_err(|e| error!(CCIHSError::WormholeError(e.to_string())))?;

        Ok(())
    }

    fn receive_message(&self, vaa_account: &AccountInfo) -> Result<CrossChainMessage> {
        let vaa_data = vaa_account.try_borrow_data()?;
        let vaa = parse_vaa(&vaa_data)
            .map_err(|e| error!(CCIHSError::WormholeError(format!("Failed to parse VAA: {}", e))))?;
        self.deserialize_message(&vaa.payload)
    }

    fn verify_message(&self, vaa_account: &AccountInfo) -> Result<bool> {
        let vaa_data = vaa_account.try_borrow_data()?;
        let vaa = parse_vaa(&vaa_data)
            .map_err(|e| error!(CCIHSError::WormholeError(format!("Failed to parse VAA: {}", e))))?;
        
        // This is a simplified check. In a real implementation, you'd need to verify
        // against the current guardian set and check other VAA properties.
        verify_signature(&vaa)
            .map_err(|e| error!(CCIHSError::WormholeError(format!("Failed to verify VAA signature: {}", e))))
    }

    fn supported_chains(&self) -> Vec<ChainId> {
        vec![ChainId::SOLANA, ChainId::ETHEREUM]
    }
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