use bytes::Bytes;
use commonware_cryptography::Signature;

type BlockHash = [u8; 32];

pub enum Message {
    // Block Notarization
    Propose {
        epoch: u64,
        view: u64,
        block: Block,
        payload: Bytes,
        signature: Signature,
    },
    Vote {
        epoch: u64,
        view: u64,
        block: BlockHash,
        signature: Signature,
    },
    Finalize {
        epoch: u64,
        view: u64,
        block: BlockHash,
        notarization: Bytes,
        signature: Signature,
    },

    // View Change
    Advance {
        epoch: u64,
        view: u64,
        block: BlockHash,
        notarization: Bytes,
    },
    Lock {
        epoch: u64,
        view: u64,
        block: BlockHash,
        notarization: Bytes,
        finalization: Bytes,
    },
    // TODO: backfill (propose + lock + seed)?
    // start with full sync and in the future add state sync

    // Beacon
    Seed {
        epoch: u64,
        view: u64,
        signature: Signature,
    },
}

#[derive(Clone)]
pub struct Mailbox {
    sender: mpsc::Sender<Message>,
}

impl Mailbox {
    pub fn new(sender: mpsc::Sender<Message>) -> Self {
        Self { sender }
    }
}
