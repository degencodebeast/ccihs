use crate::types::{ChainId, ProtocolType};
use crate::config::protocol_config::ProtocolConfigTrait;
use std::collections::{HashSet, HashMap};
use crate::protocols::wormhole::state::WormholeAddresses;
use std::rc::Rc;
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

pub struct WormholeClient {
    pub program: Program,
    pub payer: Rc<Keypair>,
    pub config: WormholeClientConfig,
}

#[derive(Clone, Debug)]
pub struct WormholeClientConfig {
    // pub rpc_endpoint: String,
    // pub commitment: CommitmentConfig,
    //pub addresses: WormholeAddresses,
    // pub supported_chains: HashSet<ChainId>,
    // pub additional_params: HashMap<String, String>,
    pub relayer_fee: u64,
    pub relayer_fee_precision: u32,
}

impl ProtocolConfigTrait for WormholeClientConfig {
    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::Wormhole
    }

    fn supported_chains(&self) -> &HashSet<ChainId> {
        &self.supported_chains
    }

    fn additional_params(&self) -> &HashMap<String, String> {
        &self.additional_params
    }
}