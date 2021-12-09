use super::*;
use frame_support::weights::Weight;

pub mod deprecated {
    use crate::Config;
    use frame_support::{decl_module};
    use sp_std::prelude::*;

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::Origin { }
    }
}

pub fn migrate_node_contracts<T: Config>() -> frame_support::weights::Weight {
    frame_support::debug::RuntimeLogger::init();
    
    let version = PalletVersion::get();
    if version != types::PalletStorageVersion::V1 {
        frame_support::debug::info!(" >>> Unused migration!");
        return 0
    }

    frame_support::debug::info!(" >>> Starting migration");

    // save number of read writes
    let mut read_writes = 0;

    // Clear old storage map
    for (node_id, state, _) in NodeContracts::iter() {
        frame_support::debug::info!(" >>> Clearing old NodeContracts map with node_id {:?}", node_id);
        let contracts: Vec<types::Contract> = Vec::new();
        NodeContracts::insert(node_id, state, contracts);
        read_writes+=1;
    };

    // Populate new storage map
    for (_, contract) in Contracts::iter() {
        if contract.state == types::ContractState::Created {
            match contract.contract_type {
                types::ContractData::NodeContract(node_contract) => {
                    let mut active_node_contracts = ActiveNodeContracts::get(node_contract.node_id);
                    active_node_contracts.push(contract.contract_id);
                    ActiveNodeContracts::insert(node_contract.node_id, active_node_contracts);
                    frame_support::debug::info!(" >>> Inserted contract id: {:?} in map with node_id: {:?}", contract.contract_id, node_contract.node_id);
                    read_writes+=1;
                },
                _ => (),
            }
        }
    };

    // Update pallet version to V2
    PalletVersion::put(types::PalletStorageVersion::V2);

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(read_writes as Weight + 1, read_writes as Weight + 1)
}