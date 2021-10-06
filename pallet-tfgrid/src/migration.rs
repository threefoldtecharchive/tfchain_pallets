use super::*;
use frame_support::weights::Weight;
use codec::{Decode, Encode};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct OldNode {
    pub version: u32,
    pub id: u32,
    pub farm_id: u32,
    pub twin_id: u32,
    pub resources: super::types::Resources,
    pub location: super::types::Location,
    pub country: Vec<u8>,
    pub city: Vec<u8>,
    // optional public config
    pub public_config: Option<super::types::PublicConfig>,
    pub uptime: u64,
    pub created: u64,
    pub farming_policy_id: u32,
}

pub mod deprecated {
    use crate::Config;
    use frame_support::{decl_module, decl_storage};
    use sp_std::prelude::*;

    decl_storage! {
        trait Store for Module<T: Config> as MyNicks {
            pub Nodes get(fn nodes): map hasher(blake2_128_concat) u32 => super::OldNode;
        }
    }
    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::Origin { }
    }
}

pub fn migrate_to_v2<T: Config>() -> frame_support::weights::Weight {
    frame_support::debug::RuntimeLogger::init();

    // Storage migrations should use storage versions for safety.
    if PalletVersion::get() == super::types::StorageVersion::V1Bytes {
        // Very inefficient, mostly here for illustration purposes.
        let count = deprecated::Nodes::iter().count();
        frame_support::debug::info!(" >>> Updating Nodes storage. Migrating {} nodes...", count);

        // We transform the storage values from the old into the new format.
        Nodes::translate::<(u32, super::types::Node), _>(
            |k: u32, (_, node): (u32, super::types::Node)| {
                frame_support::debug::info!("     Migrated node for {:?}...", k);

                let new_node = super::types::Node {
                    version: node.version,
                    id: node.id,
                    farm_id: node.farm_id,
                    twin_id: node.twin_id,
                    resources: node.resources,
                    location: node.location,
                    country: node.country,
                    city: node.city,
                    public_config: node.public_config,
                    uptime: node.uptime,
                    created: node.created,
                    farming_policy_id: node.farming_policy_id,
                    interfaces: Vec::new()
                };

                Some(new_node)
            }
        );

        // Update storage version.
        PalletVersion::put(super::types::StorageVersion::V2Struct);
        // Very inefficient, mostly here for illustration purposes.
        let count = Nodes::iter().count();
        frame_support::debug::info!(" <<< Pallet tfgrid storage updated! Migrated {} nodes ✅", count);

        // Return the weight consumed by the migration.
        T::DbWeight::get().reads_writes(count as Weight + 1, count as Weight + 1)
    } else {
        frame_support::debug::info!(" >>> Unused migration!");
        0
    }
}