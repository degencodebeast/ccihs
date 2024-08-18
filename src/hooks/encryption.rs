use super::Hook;
use crate::types::{CrossChainMessage, ChainId, CCIHSResult};
use crate::error::CCIHSError;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub struct EncryptionHook {
    key: [u8; 32],
}

impl EncryptionHook {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }
}

impl Hook for EncryptionHook {
    fn execute(&self, message: &mut CrossChainMessage, _source_chain: ChainId, _destination_chain: ChainId) -> CCIHSResult<()> {
        let key = Key::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&message.nonce.to_be_bytes());

        let encrypted_payload = cipher.encrypt(nonce, message.payload.as_ref())
            .map_err(|_| CCIHSError::EncryptionError)?;

        message.payload = encrypted_payload;
        Ok(())
    }
}



// use crypto::symmetriccipher::encrypt;
// use crypto::aes::ecb_encryptor;
// use crypto::blockmodes::NoPadding;
// use crypto::buffer::{RefReadBuffer, RefWriteBuffer};

// pub struct EncryptionHook {
//     key: [u8; 32],
// }

// impl Hook for EncryptionHook {
//     fn execute(&self, message: &mut CrossChainMessage, _source_chain: ChainId, _destination_chain: ChainId) -> CCIHSResult<()> {
//         let mut encryptor = ecb_encryptor(crypto::aes::KeySize::KeySize256, &self.key, NoPadding);
//         let mut final_result = Vec::<u8>::new();
//         let mut read_buffer = RefReadBuffer::new(&message.payload);
//         let mut buffer = [0; 4096];
//         let mut write_buffer = RefWriteBuffer::new(&mut buffer);

//         loop {
//             let result = encryptor.encrypt(&mut read_buffer, &mut write_buffer, true)?;
//             final_result.extend(write_buffer.take_read_buffer().take_remaining().iter().clone());
//             match result {
//                 BufferResult::BufferUnderflow => break,
//                 BufferResult::BufferOverflow => { }
//             }
//         }

//         message.payload = final_result;
//         Ok(())
//     }
// }