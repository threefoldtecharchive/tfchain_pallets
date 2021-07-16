#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, traits::Get,
};
use frame_system::{self as system, ensure_signed};

use hex::FromHex;

use codec::Encode;
use sp_std::prelude::*;

#[cfg(test)]
mod tests;

pub mod types;

pub trait Config: system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

// Version constant that referenced the struct version
pub const TFGRID_VERSION: u32 = 1;

decl_storage! {
    trait Store for Module<T: Config> as TfgridModule {
        pub Farms get(fn farms): map hasher(blake2_128_concat) u32 => types::Farm;
        pub FarmsByNameID get(fn farms_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u32;

        pub Nodes get(fn nodes): map hasher(blake2_128_concat) u32 => types::Node<T::AccountId>;
        pub NodesByPubkeyID get(fn nodes_by_pubkey_id): map hasher(blake2_128_concat)  T::AccountId => u32;

        pub Entities get(fn entities): map hasher(blake2_128_concat) u32 => types::Entity<T::AccountId>;
        pub EntitiesByPubkeyID get(fn entities_by_pubkey_id): map hasher(blake2_128_concat) T::AccountId => u32;
        pub EntitiesByNameID get(fn entities_by_name_id): map hasher(blake2_128_concat) Vec<u8> => u32;

        pub Twins get(fn twins): map hasher(blake2_128_concat) u32 => types::Twin<T::AccountId>;
        pub TwinsByPubkeyID get(fn twin_ids_by_pubkey): map hasher(blake2_128_concat) T::AccountId => u32;

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
        AccountId = <T as frame_system::Config>::AccountId,
    {
        FarmStored(types::Farm),
        FarmUpdated(types::Farm),
        FarmDeleted(u32),

        NodeStored(types::Node<AccountId>),
        NodeUpdated(types::Node<AccountId>),
        NodeDeleted(u32),

        EntityStored(types::Entity<AccountId>),
        EntityUpdated(types::Entity<AccountId>),
        EntityDeleted(u32),

        TwinStored(types::Twin<AccountId>),
        TwinUpdated(types::Twin<AccountId>),

        TwinEntityStored(u32, u32, Vec<u8>),
        TwinEntityRemoved(u32, u32),
        TwinDeleted(u32),

        PricingPolicyStored(types::PricingPolicy),
        CertificationCodeStored(types::CertificationCodes),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        NoneValue,
        StorageOverflow,

        CannotCreateNode,
        NodeNotExists,
        NodeWithPubkeyExists,
        CannotDeleteNode,
        NodeDeleteNotAuthorized,

        FarmExists,
        FarmNotExists,
        CannotCreateFarmWrongTwin,
        CannotUpdateFarmWrongTwin,
        CannotDeleteFarm,
        CannotDeleteFarmWrongTwin,
        IpExists,
        IpNotExists,

        EntityWithNameExists,
        EntityWithPubkeyExists,
        EntityNotExists,
        EntitySignatureDoesNotMatch,
        EntityWithSignatureAlreadyExists,
        CannotUpdateEntity,
        CannotDeleteEntity,
        SignatureLenghtIsIncorrect,

        TwinExists,
        TwinNotExists,
        TwinWithPubkeyExists,
        CannotCreateTwin,
        UnauthorizedToUpdateTwin,

        PricingPolicyExists,
        PricingPolicyNotExists,

        CertificationCodeExists,

        OffchainSignedTxError,
        NoLocalAcctForSigning
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn create_farm(origin, name: Vec<u8>, pricing_policy_id: u32, certification_type: types::CertificationType, country_id: u32, city_id: u32, public_ips: Vec<types::PublicIP>) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(!FarmsByNameID::contains_key(name.clone()), Error::<T>::FarmExists);
            ensure!(TwinsByPubkeyID::<T>::contains_key(&address), Error::<T>::TwinNotExists);
            let twin_id = TwinsByPubkeyID::<T>::get(&address);
            let twin = Twins::<T>::get(twin_id);
            ensure!(twin.address == address, Error::<T>::CannotCreateFarmWrongTwin);

            let mut id = FarmID::get();
            id = id+1;

            // reset all public ip contract id's
            // just a safeguard
            let mut pub_ips = Vec::new();
            for ip in public_ips {
                pub_ips.push(types::PublicIP{
                    ip: ip.ip,
                    gateway: ip.gateway,
                    contract_id: 0
                });
            };

            let new_farm = types::Farm {
                version: TFGRID_VERSION,
                id,
                twin_id,
                name,
                pricing_policy_id,
                certification_type,
                country_id,
                city_id,
                public_ips: pub_ips,
            };

            Farms::insert(id, &new_farm);
            FarmsByNameID::insert(new_farm.name.clone(), id);
            FarmID::put(id);

            Self::deposit_event(RawEvent::FarmStored(new_farm));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn update_farm(origin, id: u32, name: Vec<u8>, pricing_policy_id: u32, certification_type: types::CertificationType, country_id: u32, city_id: u32) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(Farms::contains_key(id), Error::<T>::FarmNotExists);

            ensure!(TwinsByPubkeyID::<T>::contains_key(&address), Error::<T>::TwinNotExists);
            let twin_id = TwinsByPubkeyID::<T>::get(&address);
            let twin = Twins::<T>::get(twin_id);
            ensure!(twin.address == address, Error::<T>::CannotUpdateFarmWrongTwin);

            let mut stored_farm = Farms::get(id);
            // Remove stored farm by name and insert new one
            FarmsByNameID::remove(stored_farm.name);

            stored_farm.name = name.clone();
            stored_farm.pricing_policy_id = pricing_policy_id;
            stored_farm.certification_type = certification_type;
            stored_farm.country_id = country_id;
            stored_farm.city_id = city_id;

            Farms::insert(id, &stored_farm);
            FarmsByNameID::insert(name, stored_farm.id);

            Self::deposit_event(RawEvent::FarmUpdated(stored_farm));
            
            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn add_farm_ip(origin, id: u32, ip: Vec<u8>, gateway: Vec<u8>) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(Farms::contains_key(id), Error::<T>::FarmNotExists);
            let mut stored_farm = Farms::get(id);

            let twin = Twins::<T>::get(stored_farm.twin_id);
            ensure!(twin.address == address, Error::<T>::CannotUpdateFarmWrongTwin);

            let new_ip = types::PublicIP {
                ip,
                gateway,
                contract_id: 0
            };

            match stored_farm.public_ips.binary_search(&new_ip) {
                Ok(_) => Err(Error::<T>::IpExists.into()),
                // If the search fails, the caller is not a member of the list
                Err(_) => {
                    stored_farm.public_ips.push(new_ip);
                    Farms::insert(stored_farm.id, &stored_farm);
                    Self::deposit_event(RawEvent::FarmUpdated(stored_farm));
                    Ok(())
                }
            }
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn remove_farm_ip(origin, id: u32, ip: Vec<u8>) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(Farms::contains_key(id), Error::<T>::FarmNotExists);
            let mut stored_farm = Farms::get(id);

            let twin = Twins::<T>::get(stored_farm.twin_id);
            ensure!(twin.address == address, Error::<T>::CannotUpdateFarmWrongTwin);

            match stored_farm.public_ips.iter().position(|pubip| pubip.ip == ip && pubip.contract_id == 0) {
                Some(index) => {
                    stored_farm.public_ips.remove(index);
                    Farms::insert(stored_farm.id, &stored_farm);
                    Self::deposit_event(RawEvent::FarmUpdated(stored_farm));
                    Ok(())
                },
                None => Err(Error::<T>::IpNotExists.into()),
            }
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
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

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn create_node(origin, farm_id: u32, resources: types::Resources, location: types::Location, country_id: u32, city_id: u32, role: types::Role, public_config: Option<types::PublicConfig>) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(Farms::contains_key(farm_id), Error::<T>::FarmNotExists);
            ensure!(!NodesByPubkeyID::<T>::contains_key(address.clone()), Error::<T>::NodeWithPubkeyExists);

            ensure!(TwinsByPubkeyID::<T>::contains_key(&address), Error::<T>::TwinNotExists);
            let twin_id = TwinsByPubkeyID::<T>::get(&address);

            let mut id = NodeID::get();
            id = id+1;

            let new_node = types::Node {
                version: TFGRID_VERSION,
                id,
                farm_id,
                twin_id,
                resources,
                location,
                country_id,
                city_id,
                address,
                role,
                public_config
            };

            Nodes::<T>::insert(id, &new_node);
            NodeID::put(id);
            NodesByPubkeyID::<T>::insert(new_node.address.clone(), id);

            Self::deposit_event(RawEvent::NodeStored(new_node));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn update_node(origin, farm_id: u32, resources: types::Resources, location: types::Location, country_id: u32, city_id: u32, role: types::Role, public_config: Option<types::PublicConfig>) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(NodesByPubkeyID::<T>::contains_key(&address), Error::<T>::NodeNotExists);
            ensure!(TwinsByPubkeyID::<T>::contains_key(&address), Error::<T>::TwinNotExists);
            let node_id = NodesByPubkeyID::<T>::get(&address);

            ensure!(Nodes::<T>::contains_key(node_id), Error::<T>::NodeNotExists);
            ensure!(Farms::contains_key(farm_id), Error::<T>::FarmNotExists);
            
            let mut stored_node = Nodes::<T>::get(node_id);

            stored_node.farm_id = farm_id;
            stored_node.resources = resources;
            stored_node.location = location;
            stored_node.country_id = country_id;
            stored_node.city_id = city_id;
            stored_node.role = role;
            stored_node.public_config = public_config;

            // override node in storage
            Nodes::<T>::insert(stored_node.id, &stored_node);

            Self::deposit_event(RawEvent::NodeUpdated(stored_node));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn delete_node(origin, id: u32) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(Nodes::<T>::contains_key(id), Error::<T>::NodeNotExists);

            let stored_node = Nodes::<T>::get(id);
            ensure!(stored_node.address == address, Error::<T>::NodeDeleteNotAuthorized);

            Nodes::<T>::remove(id);
            NodesByPubkeyID::<T>::remove(address.clone());

            Self::deposit_event(RawEvent::NodeDeleted(id));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn create_entity(origin, target: T::AccountId, name: Vec<u8>, country_id: u32, city_id: u32, signature: Vec<u8>) -> dispatch::DispatchResult {
            let _ = ensure_signed(origin)?;

            ensure!(!EntitiesByNameID::contains_key(&name), Error::<T>::EntityWithNameExists);
            ensure!(!EntitiesByPubkeyID::<T>::contains_key(&target), Error::<T>::EntityWithPubkeyExists);

            let entity_pubkey_ed25519 = Self::convert_account_to_ed25519(target.clone());

            ensure!(signature.len() == 128, Error::<T>::SignatureLenghtIsIncorrect);
            let decoded_signature_as_byteslice = <[u8; 64]>::from_hex(signature.clone()).expect("Decoding failed");

            // Decode signature into a ed25519 signature
            let ed25519_signature = sp_core::ed25519::Signature::from_raw(decoded_signature_as_byteslice);

            let mut message = Vec::new();
            message.extend_from_slice(&name);
            message.extend_from_slice(&country_id.to_be_bytes());
            message.extend_from_slice(&city_id.to_be_bytes());

            ensure!(sp_io::crypto::ed25519_verify(&ed25519_signature, &message, &entity_pubkey_ed25519), Error::<T>::EntitySignatureDoesNotMatch);

			let mut id = EntityID::get();
            id = id+1;

            let entity = types::Entity::<T::AccountId> {
                version: TFGRID_VERSION,
                id,
                name: name.clone(),
                country_id,
                city_id,
                address: target.clone(),
            };

            Entities::<T>::insert(&id, &entity);
            EntitiesByNameID::insert(&name, id);
            EntitiesByPubkeyID::<T>::insert(&target, id);
            EntityID::put(id);

            Self::deposit_event(RawEvent::EntityStored(entity));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn update_entity(origin, name: Vec<u8>, country_id: u32, city_id: u32) -> dispatch::DispatchResult {
            let pub_key = ensure_signed(origin)?;

            ensure!(!EntitiesByNameID::contains_key(&name), Error::<T>::EntityWithNameExists);

            ensure!(EntitiesByPubkeyID::<T>::contains_key(&pub_key), Error::<T>::EntityNotExists);
            let stored_entity_id = EntitiesByPubkeyID::<T>::get(&pub_key);

            ensure!(Entities::<T>::contains_key(&stored_entity_id), Error::<T>::EntityNotExists);
            let mut stored_entity = Entities::<T>::get(stored_entity_id);

            ensure!(stored_entity.address == pub_key, Error::<T>::CannotUpdateEntity);

            // remove entity by name id
            EntitiesByNameID::remove(&stored_entity.name);
            
            stored_entity.name = name.clone();
            stored_entity.country_id = country_id;
            stored_entity.city_id = city_id;

            // overwrite entity
            Entities::<T>::insert(&stored_entity_id, &stored_entity);

            // re-insert with new name
            EntitiesByNameID::insert(&name, stored_entity_id);

            Self::deposit_event(RawEvent::EntityUpdated(stored_entity));

            Ok(())
        }

        // TODO: delete all object that have an entity id reference?
        #[weight = 10 + T::DbWeight::get().writes(1)]
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

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn create_twin(origin, ip: Vec<u8>) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;

            ensure!(!TwinsByPubkeyID::<T>::contains_key(&address), Error::<T>::TwinWithPubkeyExists);

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
			TwinsByPubkeyID::<T>::insert(&address.clone(), twin_id);

			Self::deposit_event(RawEvent::TwinStored(twin));
			
			Ok(())
        }
        
        #[weight = 10 + T::DbWeight::get().writes(1)]
		pub fn update_twin(origin, ip: Vec<u8>) -> dispatch::DispatchResult {
            let address = ensure_signed(origin)?;
            
            ensure!(TwinsByPubkeyID::<T>::contains_key(address.clone()), Error::<T>::TwinNotExists);
            let twin_id = TwinsByPubkeyID::<T>::get(address.clone());
            let mut twin = Twins::<T>::get(&twin_id);

            // Make sure only the owner of this twin can update his twin
            ensure!(twin.address == address, Error::<T>::UnauthorizedToUpdateTwin);

            twin.ip = ip.clone();

            Twins::<T>::insert(&twin_id, &twin);

            Self::deposit_event(RawEvent::TwinUpdated(twin));
            Ok(())
		}

        // Method for twins only
        #[weight = 10 + T::DbWeight::get().writes(1)]
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

            let entity_pubkey_ed25519 = Self::convert_account_to_ed25519(stored_entity.address.clone());

            let mut message = Vec::new();

            message.extend_from_slice(&entity_id.to_be_bytes());
            message.extend_from_slice(&twin_id.to_be_bytes());

            ensure!(sp_io::crypto::ed25519_verify(&ed25519_signature, &message, &entity_pubkey_ed25519), Error::<T>::EntitySignatureDoesNotMatch);

            // Store proof
            twin.entities.push(entity_proof);

            // Update twin
            Twins::<T>::insert(&twin_id, &twin);

            Self::deposit_event(RawEvent::TwinEntityStored(twin_id, entity_id, signature));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
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

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn delete_twin(origin, twin_id: u32) -> dispatch::DispatchResult {
            let pub_key = ensure_signed(origin)?;

            ensure!(Twins::<T>::contains_key(&twin_id), Error::<T>::TwinNotExists);

            let twin = Twins::<T>::get(&twin_id);
            // Make sure only the owner of this twin can call this method
            ensure!(twin.address == pub_key, Error::<T>::UnauthorizedToUpdateTwin);

            Twins::<T>::remove(&twin_id);

            // remove twin id from this users map of twin ids
            TwinsByPubkeyID::<T>::remove(&pub_key.clone());

            Self::deposit_event(RawEvent::TwinDeleted(twin_id));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn create_pricing_policy(origin, name: Vec<u8>, unit: types::Unit, su: u32, cu: u32, nu: u32, ipu: u32) -> dispatch::DispatchResult {
            let _ = ensure_signed(origin)?;

            ensure!(!PricingPoliciesByNameID::contains_key(&name), Error::<T>::PricingPolicyExists);

            let mut id = PricingPolicyID::get();
            id = id+1;


            let new_policy = types::PricingPolicy {
                version: TFGRID_VERSION,
                id,
                name,
                unit,
                su,
                cu,
                nu,
                ipu
            };

            PricingPolicies::insert(&id, &new_policy);
            PricingPoliciesByNameID::insert(&new_policy.name, &id);
            PricingPolicyID::put(id);

            Self::deposit_event(RawEvent::PricingPolicyStored(new_policy));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
        pub fn update_pricing_policy(origin, id: u32, name: Vec<u8>, unit: types::Unit, su: u32, cu: u32, nu: u32, ipu: u32) -> dispatch::DispatchResult {
            let _ = ensure_signed(origin)?;

            ensure!(PricingPolicies::contains_key(&id), Error::<T>::PricingPolicyNotExists);
            ensure!(!PricingPoliciesByNameID::contains_key(&name), Error::<T>::PricingPolicyExists);
            let mut pricing_policy = PricingPolicies::get(id);

            if name != pricing_policy.name {
                PricingPoliciesByNameID::remove(&pricing_policy.name);
            }

            pricing_policy.name = name;
            pricing_policy.unit = unit;
            pricing_policy.su = su;
            pricing_policy.cu = cu;
            pricing_policy.nu = nu;
            pricing_policy.ipu = ipu;

            PricingPolicies::insert(&id, &pricing_policy);
            PricingPoliciesByNameID::insert(&pricing_policy.name, &id);
            PricingPolicyID::put(id);

            Self::deposit_event(RawEvent::PricingPolicyStored(pricing_policy));

            Ok(())
        }

        #[weight = 10 + T::DbWeight::get().writes(1)]
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

            Self::deposit_event(RawEvent::CertificationCodeStored(certification_code));

            Ok(())
        }
    }
}

impl<T: Config> Module<T> {
    pub fn convert_account_to_ed25519(account: T::AccountId) -> sp_core::ed25519::Public {
        // Decode entity's public key
        let account_vec = &account.encode();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&account_vec);
        let ed25519_pubkey = sp_core::ed25519::Public::from_raw(bytes);

        return ed25519_pubkey;
    }
}
