#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, traits::Get,
};
use frame_system::{self as system, ensure_signed};

use hex::FromHex;

use codec::Encode;
use sp_std::prelude::*;

#[cfg(test)]
mod tests;

mod types;

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

// Version constant that referenced the struct version
pub const TFGRID_VERSION: u32 = 1;

decl_storage! {
    trait Store for Module<T: Trait> as TfgridModule {
        pub Farms get(fn farms): map hasher(blake2_128_concat) u32 => types::Farm;
        pub FarmsByNameID get(fn farms_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u32;

        pub Nodes get(fn nodes): map hasher(blake2_128_concat) u32 => types::Node<T::AccountId>;
        pub NodesByPubkeyID get(fn nodes_by_pubkey_id): map hasher(blake2_128_concat) Vec<u8> => u32;

        pub Entities get(fn entities): map hasher(blake2_128_concat) u32 => types::Entity<T::AccountId>;
        pub EntitiesByPubkeyID get(fn entities_by_pubkey_id): map hasher(blake2_128_concat) T::AccountId => u32;
        pub EntitiesByNameID get(fn entities_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u32;

        pub Twins get(fn twins): map hasher(blake2_128_concat) u32 => types::Twin<T::AccountId>;
        pub TwinsByPubkey get(fn twin_ids_by_pubkey): map hasher(blake2_128_concat) T::AccountId => Vec<u32>;

        pub PricingPolicies get(fn pricing_policies): map hasher(blake2_128_concat) u32 => types::PricingPolicy;
        pub PricingPoliciesByNameID get(fn pricing_policies_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u32;

        pub CertificationCodes get(fn certification_codes): map hasher(blake2_128_concat) u32 => types::CertificationCodes;
        pub CertificationCodesByNameID get(fn certification_codes_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u32;

        // ID maps
        FarmID: u32;
        NodeID: u32;
        EntityID: u32;
        TwinID: u32;
        PricingPolicyID: u32;
        CertificationCodeID: u32;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        FarmStored(
            u32,
            u32,
            Vec<u8>,
            u32,
            u32,
            u32,
            u32,
            types::CertificationType,
        ),
        FarmDeleted(u32),

        NodeStored(u32, u32, u32, types::Resources, types::Location, u32, u32, Vec<u8>, AccountId, types::Role),
        NodeDeleted(u32),

        EntityStored(u32, u32, Vec<u8>, u32, u32, AccountId),
        EntityUpdated(u32, Vec<u8>, u32, u32, AccountId),
        EntityDeleted(u32),

        TwinStored(u32, u32, AccountId, Vec<u8>),
        TwinUpdated(u32, AccountId, Vec<u8>),

        TwinEntityStored(u32, u32, Vec<u8>),
        TwinEntityRemoved(u32, u32),
        TwinDeleted(u32),

        PricingPolicyStored(u32, Vec<u8>, u32),
        CertificationCodeStored(u32, Vec<u8>, u32),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        NoneValue,
        StorageOverflow,

        CannotCreateNode,
        NodeNotExists,
        NodeWithPubkeyExists,
        CannotDeleteNode,

        FarmExists,
        FarmNotExists,
        CannotCreateFarmWrongTwin,
        CannotDeleteFarm,
        CannotDeleteFarmWrongTwin,

        EntityWithNameExists,
        EntityWithPubkeyExists,
        EntityNotExists,
        EntitySignatureDoesNotMatch,
        EntityWithSignatureAlreadyExists,
        CannotUpdateEntity,
        CannotDeleteEntity,

        TwinExists,
        TwinNotExists,
        CannotCreateTwin,
        UnauthorizedToUpdateTwin,

        PricingPolicyExists,

        CertificationCodeExists,

        OffchainSignedTxError,
        NoLocalAcctForSigning
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create_farm(origin, farm: types::Farm) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(!FarmsByNameID::contains_key(farm.name.clone()), Error::<T>::FarmExists);

            ensure!(Twins::<T>::contains_key(farm.twin_id), Error::<T>::TwinNotExists);
            
            let twin = Twins::<T>::get(farm.twin_id);
            ensure!(twin.address == address, Error::<T>::CannotCreateFarmWrongTwin);

            let mut id = FarmID::get();
            id = id+1;

            let mut new_farm = farm.clone();

            new_farm.id = id;
            new_farm.version = TFGRID_VERSION;

            Farms::insert(id, &new_farm);
            FarmsByNameID::insert(new_farm.name.clone(), id);
            FarmID::put(id);

            Self::deposit_event(RawEvent::FarmStored(
                TFGRID_VERSION,
                id,
                new_farm.name,
                new_farm.twin_id,
                new_farm.pricing_policy_id,
                new_farm.country_id,
                new_farm.city_id,
                new_farm.certification_type
            ));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn delete_farm(origin, id: u32) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(Farms::contains_key(id), Error::<T>::FarmNotExists);
            let stored_farm = Farms::get(id);

            let twin = Twins::<T>::get(stored_farm.twin_id);
            ensure!(twin.address == address, Error::<T>::CannotDeleteFarmWrongTwin);

            // delete farm
            Farms::remove(id);

            // Remove stored farm by name and insert new one
            FarmsByNameID::remove(stored_farm.name);

            Self::deposit_event(RawEvent::FarmDeleted(id));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create_node(origin, node: types::Node<T::AccountId>) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(Farms::contains_key(node.farm_id), Error::<T>::FarmNotExists);
            ensure!(!NodesByPubkeyID::contains_key(node.pub_key.clone()), Error::<T>::NodeWithPubkeyExists);

            let mut id = NodeID::get();
            id = id+1;

            let mut new_node = node.clone();
            new_node.id = id;
            new_node.address = address.clone();
            new_node.version = TFGRID_VERSION;

            Nodes::<T>::insert(id, &new_node);
            NodeID::put(id);
            NodesByPubkeyID::insert(node.pub_key.clone(), id);

            Self::deposit_event(RawEvent::NodeStored(
                TFGRID_VERSION,
                id,
                new_node.farm_id,
                new_node.resources,
                new_node.location,
                new_node.country_id,
                new_node.city_id,
                new_node.pub_key,
                new_node.address,
                new_node.role
            ));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn delete_node(origin, id: u32) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(Nodes::<T>::contains_key(id), Error::<T>::NodeNotExists);

            let stored_node = Nodes::<T>::get(id);
            ensure!(stored_node.address == address, Error::<T>::NodeNotExists);

            Nodes::<T>::remove(id);
            NodesByPubkeyID::remove(stored_node.pub_key.clone());

            Self::deposit_event(RawEvent::NodeDeleted(id));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create_entity(origin, name: Vec<u8>, country_id: u32, city_id: u32) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(!EntitiesByNameID::contains_key(&name), Error::<T>::EntityWithNameExists);

            ensure!(!EntitiesByPubkeyID::<T>::contains_key(&address), Error::<T>::EntityWithPubkeyExists);

			let mut id = EntityID::get();
            id = id+1;

            let entity = types::Entity::<T::AccountId> {
                version: TFGRID_VERSION,
                id,
                name: name.clone(),
                country_id,
                city_id,
                address: address.clone(),
            };

            Entities::<T>::insert(&id, &entity);
            EntitiesByNameID::insert(&name, id);
            EntitiesByPubkeyID::<T>::insert(&address, id);
            EntityID::put(id);

            Self::deposit_event(RawEvent::EntityStored(TFGRID_VERSION, id, name, country_id, city_id, address));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn update_entity(origin, name: Vec<u8>, country_id: u32, city_id: u32) -> dispatch::DispatchResult {
            let pub_key = ensure_signed(origin)?;

            ensure!(EntitiesByPubkeyID::<T>::contains_key(&pub_key), Error::<T>::EntityNotExists);
            let stored_entity_id = EntitiesByPubkeyID::<T>::get(&pub_key);

            ensure!(Entities::<T>::contains_key(&stored_entity_id), Error::<T>::EntityNotExists);
            let stored_entity = Entities::<T>::get(stored_entity_id);

            ensure!(stored_entity.address == pub_key, Error::<T>::CannotUpdateEntity);

            let entity = types::Entity::<T::AccountId> {
                version: TFGRID_VERSION,
                id: stored_entity_id,
                name: name.clone(),
                country_id,
                city_id,
                address: pub_key.clone(),
            };

            // overwrite entity
            Entities::<T>::insert(&stored_entity_id, &entity);

            // remove entity by name id
            EntitiesByNameID::remove(&stored_entity.name);
            // re-insert with new name
            EntitiesByNameID::insert(&name, stored_entity_id);

            Self::deposit_event(RawEvent::EntityUpdated(stored_entity_id, name, country_id, city_id, pub_key));

            Ok(())
        }

        // TODO: delete all object that have an entity id reference?
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn delete_entity(origin) -> dispatch::DispatchResult {
            let pub_key = ensure_signed(origin)?;

            ensure!(EntitiesByPubkeyID::<T>::contains_key(&pub_key), Error::<T>::EntityNotExists);
            let stored_entity_id = EntitiesByPubkeyID::<T>::get(&pub_key);

            ensure!(Entities::<T>::contains_key(&stored_entity_id), Error::<T>::EntityNotExists);
            let stored_entity = Entities::<T>::get(stored_entity_id);

            ensure!(stored_entity.address == pub_key, Error::<T>::CannotDeleteEntity);

            // Remove entity from storage
            Entities::<T>::remove(&stored_entity_id);

            // remove entity by name id
            EntitiesByNameID::remove(&stored_entity.name);

            // remove entity by pubkey id
            EntitiesByPubkeyID::<T>::remove(&pub_key);

            Self::deposit_event(RawEvent::EntityDeleted(stored_entity_id));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create_twin(origin, ip: Vec<u8>) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            let mut twin_id = TwinID::get();
            twin_id = twin_id+1;

			let twin = types::Twin::<T::AccountId> {
                version: TFGRID_VERSION,
				id: twin_id,
				address: address.clone(),
                entities: Vec::new(),
                ip: ip.clone(),
			};

            Twins::<T>::insert(&twin_id, &twin);
            TwinID::put(twin_id);

            // add the twin id to this users map of twin ids
            let mut twins_by_pubkey = TwinsByPubkey::<T>::get(&address.clone());
            twins_by_pubkey.push(twin_id);
			TwinsByPubkey::<T>::insert(&address.clone(), twins_by_pubkey);

			Self::deposit_event(RawEvent::TwinStored(TFGRID_VERSION, twin_id, address, ip));
			
			Ok(())
        }
        
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn update_twin(origin, twin_id: u32, ip: Vec<u8>) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;
            
            ensure!(TwinsByPubkey::<T>::contains_key(address.clone()), Error::<T>::TwinNotExists);
            let twin_ids = TwinsByPubkey::<T>::get(address.clone());

            match twin_ids.binary_search(&twin_id) {
                Ok(_) => {
                    let mut twin = Twins::<T>::get(&twin_id);
                    // Make sure only the owner of this twin can update his twin
                    ensure!(twin.address == address, Error::<T>::UnauthorizedToUpdateTwin);
        
                    twin.ip = ip.clone();
        
                    Twins::<T>::insert(&twin_id, &twin);
        
                    Self::deposit_event(RawEvent::TwinUpdated(twin_id, address, ip));
                    Ok(())
                },
                Err(_) => Err(Error::<T>::TwinNotExists.into()),
            }
		}

        // Method for twins only
        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn add_twin_entity(origin, twin_id: u32, entity_id: u32, signature: Vec<u8>) -> dispatch::DispatchResult {
            let pub_key = ensure_signed(origin)?;

            ensure!(Twins::<T>::contains_key(&twin_id), Error::<T>::TwinNotExists);

            ensure!(Entities::<T>::contains_key(&entity_id), Error::<T>::EntityNotExists);
            let stored_entity = Entities::<T>::get(entity_id);

            let mut twin = Twins::<T>::get(&twin_id);
            // Make sure only the owner of this twin can call this method
            ensure!(twin.address == pub_key, Error::<T>::UnauthorizedToUpdateTwin);

            let entity_proof = types::EntityProof{
                entity_id,
                signature: signature.clone()
            };

            ensure!(!twin.entities.contains(&entity_proof), Error::<T>::EntityWithSignatureAlreadyExists);

            let decoded_signature_as_byteslice = <[u8; 64]>::from_hex(signature.clone()).expect("Decoding failed");

            // Decode signature into a ed25519 signature
            let ed25519_signature = sp_core::ed25519::Signature::from_raw(decoded_signature_as_byteslice);
            // let sr25519_signature = sp_core::sr25519::Signature::from_raw(decoded_signature_as_byteslice);

            let entity_pubkey_ed25519 = Self::convert_account_to_ed25519(stored_entity.address.clone());

            // let entity_pubkey_sr25519 = sp_core::sr25519::Public::from_raw(bytes);
            // debug::info!("Public key: {:?}", entity_pubkey_sr25519);

            let mut message = vec![];

            message.extend_from_slice(&entity_id.to_be_bytes());
            message.extend_from_slice(&twin_id.to_be_bytes());

            // Verify that the signature contains the message with the entity's public key
            let ed25519_verified = sp_io::crypto::ed25519_verify(&ed25519_signature, &message, &entity_pubkey_ed25519);

            // let sr25519_verified = sp_io::crypto::sr25519_verify(&sr25519_signature, &message, &entity_pubkey_sr25519);
            // let sr25519_verified = sr25519_signature.verify(message.as_slice(), &entity_pubkey_sr25519);
            // debug::info!("sr25519 verified? {:?}", sr25519_verified);

            ensure!(sp_io::crypto::ed25519_verify(&ed25519_signature, &message, &entity_pubkey_ed25519), Error::<T>::EntitySignatureDoesNotMatch);

            // Store proof
            twin.entities.push(entity_proof);

            // Update twin
            Twins::<T>::insert(&twin_id, &twin);

            Self::deposit_event(RawEvent::TwinEntityStored(twin_id, entity_id, signature));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn delete_twin_entity(origin, twin_id: u32, entity_id: u32) -> dispatch::DispatchResult {
            let pub_key = ensure_signed(origin)?;

            ensure!(Twins::<T>::contains_key(&twin_id), Error::<T>::TwinNotExists);

            let mut twin = Twins::<T>::get(&twin_id);
            // Make sure only the owner of this twin can call this method
            ensure!(twin.address == pub_key, Error::<T>::UnauthorizedToUpdateTwin);

            ensure!(twin.entities.iter().any(|v| v.entity_id == entity_id), Error::<T>::EntityNotExists);

            let index = twin.entities.iter().position(|x| x.entity_id == entity_id).unwrap();
            twin.entities.remove(index);

            // Update twin
            Twins::<T>::insert(&twin_id, &twin);

            Self::deposit_event(RawEvent::TwinEntityRemoved(twin_id, entity_id));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn delete_twin(origin, twin_id: u32) -> dispatch::DispatchResult {
            let pub_key = ensure_signed(origin)?;

            ensure!(Twins::<T>::contains_key(&twin_id), Error::<T>::TwinNotExists);

            let twin = Twins::<T>::get(&twin_id);
            // Make sure only the owner of this twin can call this method
            ensure!(twin.address == pub_key, Error::<T>::UnauthorizedToUpdateTwin);

            Twins::<T>::remove(&twin_id);

            // remove twin id from this users map of twin ids
            let mut twins_by_pubkey = TwinsByPubkey::<T>::get(&pub_key.clone());
            if let Some(pos) = twins_by_pubkey.iter().position(|x| *x == twin_id) {
                twins_by_pubkey.remove(pos);
                TwinsByPubkey::<T>::insert(&pub_key.clone(), twins_by_pubkey);
            }

            Self::deposit_event(RawEvent::TwinDeleted(twin_id));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create_pricing_policy(origin, name: Vec<u8>, currency: Vec<u8>, su: u32, cu: u32, nu: u32) -> dispatch::DispatchResult {
            let _ = ensure_signed(origin)?;

            ensure!(!PricingPoliciesByNameID::contains_key(&name), Error::<T>::PricingPolicyExists);

            let mut id = PricingPolicyID::get();
            id = id+1;

            let policy = types::PricingPolicy {
                version: TFGRID_VERSION,
                id,
                name: name.clone(),
                currency,
                su,
                cu,
                nu
            };

            PricingPolicies::insert(&id, &policy);
            PricingPoliciesByNameID::insert(&name, &id);
            PricingPolicyID::put(id);

            Self::deposit_event(RawEvent::PricingPolicyStored(TFGRID_VERSION, name, id));

            Ok(())
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn create_certification_code(origin, name: Vec<u8>, description: Vec<u8>, certification_code_type: types::CertificationCodeType) -> dispatch::DispatchResult {
            let _ = ensure_signed(origin)?;

            ensure!(!CertificationCodesByNameID::contains_key(&name), Error::<T>::CertificationCodeExists);

            let mut id = CertificationCodeID::get();
            id = id+1;

            let certification_code = types::CertificationCodes{
                version: TFGRID_VERSION,
                id,
                name: name.clone(),
                description,
                certification_code_type
            };

            CertificationCodes::insert(&id, &certification_code);
            CertificationCodesByNameID::insert(&name, &id);
            CertificationCodeID::put(id);

            Self::deposit_event(RawEvent::CertificationCodeStored(TFGRID_VERSION, name, id));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn convert_account_to_ed25519(account: T::AccountId) -> sp_core::ed25519::Public {
        // Decode entity's public key
        let account_vec = &account.encode();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&account_vec);
        let ed25519_pubkey = sp_core::ed25519::Public::from_raw(bytes);

        return ed25519_pubkey;
    }
}
