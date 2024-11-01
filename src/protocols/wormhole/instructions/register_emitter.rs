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

use crate::protocols::wormhole::state::{
    SenderConfig,
    RedeemerConfig,
    InboundTokenBridgeAddresses,
    OutboundTokenBridgeAddresses,
    WormholeAddresses,
    GeneralMessageConfig,
    WormholeEmitter,
    SEED_PREFIX_SENT,
    ForeignEmitter
};
use crate::protocols::wormhole::config::WormholeClientConfig;
use wormhole_anchor_sdk::{wormhole, token_bridge};
use crate::protocols::wormhole::config::WormholeClient;

impl WormholeClient {
    pub async fn register_emitter(
        &self,
        chain: u16,
        address: [u8; 32],
    ) -> Result<()> {
        // 1. Derive PDAs
        let (general_message_config_pda, _) = Pubkey::find_program_address(
            &[GeneralMessageConfig::SEED_PREFIX],
            &self.program.id(),
        );

        let (foreign_emitter_pda, foreign_emitter_bump) = Pubkey::find_program_address(
            &[
                ForeignEmitter::SEED_PREFIX,
                &chain.to_le_bytes()[..],
            ],
            &self.program.id(),
        );

        // 2. Create instruction data
        let mut instruction_data = vec![];
        
        // Add instruction discriminator for 'register_emitter'
        instruction_data.extend_from_slice(&[/* instruction discriminator */]);
        
        // Add chain and address parameters
        instruction_data.extend_from_slice(&chain.to_le_bytes());
        instruction_data.extend_from_slice(&address);

        // 3. Create the accounts vector
        let accounts = vec![
            AccountMeta::new(self.payer.pubkey(), true),              // owner (signer)
            AccountMeta::new_readonly(general_message_config_pda, false), // general_message_config
            AccountMeta::new(foreign_emitter_pda, false),             // foreign_emitter
            AccountMeta::new_readonly(system_program::id(), false),    // system_program
        ];

        // 4. Create the instruction
        let instruction = Instruction {
            program_id: self.program.id(),
            accounts,
            data: instruction_data,
        };

        // 5. Create and send transaction
        let recent_blockhash = self.program.rpc().get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer.pubkey()),
            &[&*self.payer],
            recent_blockhash,
        );

        self.program.rpc().send_and_confirm_transaction(&transaction)?;

        Ok(())
    }
}


// use anchor_lang::prelude::*;
// use wormhole_anchor_sdk::{wormhole, token_bridge};
// use crate::types::CCIHSResult;
// use crate::utility::error::CCIHSError;
// use crate::protocols::wormhole::state::{GeneralMessageConfig, ForeignEmitter};
// use crate::protocols::wormhole::error::WormholeError;

//  /// This instruction registers a new foreign emitter (from another network)
//     /// and saves the emitter information in a ForeignEmitter account. This
//     /// instruction is owner-only, meaning that only the owner of the program
//     /// (defined in the [Config] account) can add and update emitters.
//     ///
//     /// # Arguments
//     ///
//     /// * `ctx`     - `RegisterForeignEmitter` context
//     /// * `chain`   - Wormhole Chain ID
//     /// * `address` - Wormhole Emitter Address
//     pub fn register_emitter_handler(
//         ctx: Context<RegisterEmitter>,
//         chain: u16,
//         address: [u8; 32],
//     ) -> Result<()> {
//         // Foreign emitter cannot share the same Wormhole Chain ID as the
//         // Solana Wormhole program's. And cannot register a zero address.
//         require!(
//             chain > 0 && chain != wormhole::CHAIN_ID_SOLANA && !address.iter().all(|&x| x == 0),
//             WormholeError::InvalidForeignEmitter,
//         );

//         // Save the emitter info into the ForeignEmitter account.
//         let emitter = &mut ctx.accounts.foreign_emitter;
//         emitter.chain = chain;
//         emitter.address = address;

//     // Done.
//     Ok(())
// }

// #[derive(Accounts)]
// #[instruction(chain: u16)]
// pub struct RegisterEmitter<'info> {
//     #[account(mut)]
//     /// Owner of the program set in the [`Config`] account. Signer for creating
//     /// the [`ForeignEmitter`] account.
//     pub owner: Signer<'info>,

//     #[account(
//         has_one = owner, //@ HelloWorldError::OwnerOnly,
//         seeds = [GeneralMessageConfig::SEED_PREFIX],
//         bump
//     )]
//     /// Config account. This program requires that the `owner` specified in the
//     /// context equals the pubkey specified in this account. Read-only.
//     pub general_message_config: Account<'info, GeneralMessageConfig>,

//     #[account(
//         init_if_needed,
//         payer = owner,
//         seeds = [
//             ForeignEmitter::SEED_PREFIX,
//             &chain.to_le_bytes()[..]
//         ],
//         bump,
//         space = ForeignEmitter::MAXIMUM_SIZE
//     )]
//     /// Foreign Emitter account. Create this account if an emitter has not been
//     /// registered yet for this Wormhole chain ID. If there already is an
//     /// emitter address saved in this account, overwrite it.
//     pub foreign_emitter: Account<'info, ForeignEmitter>,

//     /// System program.
//     pub system_program: Program<'info, System>,
// }

