use commonware_consensus::ThresholdSupervisor;
use commonware_cryptography::{
    bls12381::primitives::{
        group::{Share, Signature},
        poly::Public,
    },
    Hasher, Scheme,
};

mod actor;
mod ingress;

pub use actor::Actor;
pub use ingress::{Mailbox, Message};

use crate::linked::View;

pub struct Config<
    C: Scheme,
    H: Hasher,
    S: ThresholdSupervisor<Seed = Signature, Index = View, Share = Share, Identity = Public>,
> {
    pub crypto: C,
    pub hasher: H,
    pub supervisor: S,
    pub mailbox_size: usize,
    pub share: Share,
    pub namespace: Vec<u8>,
}
