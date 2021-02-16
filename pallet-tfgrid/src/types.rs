#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::traits::Vec;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Farm {
    pub id: u32,
    pub name: Vec<u8>,
    pub entity_id: u32,
    pub twin_id: u32,
    pub pricing_policy_id: u32,
    pub certification_type: CertificationType,
    pub country_id: u32,
    pub city_id: u32,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Node<AccountId> {
    pub id: u32,
    pub farm_id: u32,
    pub resources: Resources,
    pub location: Location,
    pub country_id: u32,
    pub city_id: u32,
    //public key of parity
    pub address: AccountId,
    pub pub_key: sp_core::ed25519::Public,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Gateway<AccountId> {
    pub id: u32,
    pub farm_id: u32,
    pub location: Location,
    pub country_id: u32,
    pub city_id: u32,
    //public key of parity
    pub address: AccountId,
    pub pub_key: sp_core::ed25519::Public,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
pub struct Entity<AccountId> {
    pub entity_id: u32,
    pub name: Vec<u8>,
    pub country_id: u32,
    pub city_id: u32,
    pub address: AccountId,
    pub pub_key: sp_core::ed25519::Public,
}

//digital twin
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
pub struct Twin<AccountId> {
    pub twin_id: u32,
    //substrate account id = public key (32 bytes)
    //also used by PAN network
    pub address: AccountId,
    pub pub_key: sp_core::ed25519::Public,
    //link to person's or companies who own this twin
    pub entities: Vec<EntityProof>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default)]
pub struct EntityProof {
    pub entity_id: u32,
    pub signature: Vec<u8>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug, Copy)]
pub struct Resources {
    pub hru: u32,
    pub sru: u32,
    pub cru: u32,
    pub mru: u32,
}

// Store Location long and lat as string
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Location {
    pub longitude: Vec<u8>,
    pub latitude: Vec<u8>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct PricingPolicy {
    pub id: u32,
    pub name: Vec<u8>,
    pub currency: Vec<u8>,
    pub su: u32,
    pub cu: u32,
    pub nu: u32,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct CertificationCodes {
    pub id: u32,
    pub name: Vec<u8>,
    pub description: Vec<u8>,
    pub certification_code_type: CertificationCodeType,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
pub enum CertificationCodeType {
    Farm,
    Entity,
}

impl Default for CertificationCodeType {
    fn default() -> CertificationCodeType {
        CertificationCodeType::Farm
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug, Copy)]
pub enum CertificationType {
    None,
    Silver,
    Gold,
}

impl Default for CertificationType {
    fn default() -> CertificationType {
        CertificationType::None
    }
}
