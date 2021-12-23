use super::*;
use frame_support::weights::Weight;
use codec::{Decode, Encode};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct ContractV3 {
    pub version: u32,
    pub state: ContractState,
    pub contract_id: u64,
    pub twin_id: u32,
    pub contract_type: super::types::ContractData,
}

impl Default for ContractState {
    fn default() -> ContractState {
        ContractState::Created
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
pub enum ContractState {
    Created,
    Deleted,
    OutOfFunds,
}

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
    frame_support::debug::info!(" >>> Version: {:?}", version);

    if version != types::PalletStorageVersion::V2 {
        frame_support::debug::info!(" >>> Unused migration!");
        return 0
    }

    frame_support::debug::info!(" >>> Starting migration");

    // save number of read writes
    let mut read_writes = 0;

    Contracts::translate::<ContractV3, _>(
        |k, ctr| {
            frame_support::debug::info!("     Migrated contract for {:?}...", k);

            let new_state = match ctr.state {
                ContractState::Created => super::types::ContractState::Created,
                ContractState::Deleted => super::types::ContractState::Deleted(super::types::Cause::CanceledByUser),
                ContractState::OutOfFunds => super::types::ContractState::Deleted(super::types::Cause::OutOfFunds),
            };

            let new_contract = super::types::Contract {
                version: 2,
                state: new_state,
                contract_id: ctr.contract_id,
                twin_id: ctr.twin_id,
                contract_type: ctr.contract_type
            };

            read_writes+=1;
            Some(new_contract)
    });


    // Populate new storage map
    for (ctr_id, contract) in Contracts::iter() {
        if contract.state == super::types::ContractState::Deleted(super::types::Cause::OutOfFunds) {
            match contract.contract_type {
                types::ContractData::NodeContract(node_contract) => {
                    let mut active_node_contracts = ActiveNodeContracts::get(node_contract.node_id);
                    match active_node_contracts.iter().position(|x| x == &ctr_id) {
                        Some(index) => {
                            active_node_contracts.remove(index);
                            ActiveNodeContracts::insert(node_contract.node_id, active_node_contracts);
                            read_writes+=1;
                        },
                        None => {}
                    };
                },
                _ => (),
            }
        }
    };

    // Update pallet version to V3
    PalletVersion::put(types::PalletStorageVersion::V3);

    // Return the weight consumed by the migration.
    T::DbWeight::get().reads_writes(read_writes as Weight + 1, read_writes as Weight + 1)
} 