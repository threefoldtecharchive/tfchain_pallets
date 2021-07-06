#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_event, decl_module, decl_storage, decl_error, ensure, debug,
	traits::{Vec},
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::{DispatchResult};
use codec::{Decode, Encode};
use pallet_tfgrid;

pub trait Config: system::Config + pallet_tfgrid::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

decl_event!(
	pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
    {
		ContractCreated(u128, u32, Vec<u8>, u32, AccountId),
		IPsReserved(u128, Vec<Vec<u8>>),
		ContractCanceled(u128),
		IPsFreed(u128, Vec<Vec<u8>>),
		ContractDeployed(u128, AccountId),
	}
);

decl_error! {
	/// Error for the smart contract module.
	pub enum Error for Module<T: Config> {
		TwinNotExists,
		NodeNotExists,
		FarmNotExists,
		FarmHasNotEnoughPublicIPs,
		FarmHasNotEnoughPublicIPsFree,
		FailedToReserveIP,
		FailedToFreeIPs,
		ContractNotExists,
		TwinNotAuthorizedToCreateContract,
		TwinNotAuthorizedToCancelContract,
		NodeNotAuthorizedToDeployContract,
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Contract<AccountId> {
	twin_id: u32,
	node_id: AccountId,
    workload: Vec<u8>,
    public_ips: u32,
	state: ContractState
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
pub enum ContractState {
	Created,
	Deployed
}

impl Default for ContractState {
	fn default() -> ContractState {
		ContractState::Created
	}
}

decl_storage! {
	trait Store for Module<T: Config> as c {
        pub Contracts get(fn contracts): map hasher(blake2_128_concat) u128 => Contract<T::AccountId>;
        ContractID: u128;
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		fn deposit_event() = default;
		
		#[weight = 10_000]
		fn create_contract(origin, contract: Contract<T::AccountId>){
            let address = ensure_signed(origin)?;
            Self::_create_contract(address, contract)?;
		}

		#[weight = 10_000]
		fn cancel_contract(origin, contract_id: u128){
            let address = ensure_signed(origin)?;
            Self::_cancel_contract(address, contract_id)?;
		}

		#[weight = 10_000]
		fn deploy_contract(origin, contract_id: u128) {
			let address = ensure_signed(origin)?;
			Self::_deploy_contract(address, contract_id)?;
		}

		// fn on_finalize(block: T::BlockNumber) {
			
		// }
	}
}

impl<T: Config> Module<T> {
	pub fn _create_contract(address: T::AccountId, contract: Contract<T::AccountId>) -> DispatchResult {
		let mut id = ContractID::get();
		id = id+1;
		
		ensure!(pallet_tfgrid::Twins::<T>::contains_key(&contract.twin_id), Error::<T>::TwinNotExists);
		let twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id);
		ensure!(twin.address == address, Error::<T>::TwinNotAuthorizedToCreateContract);

		ensure!(pallet_tfgrid::NodesByPubkeyID::<T>::contains_key(&contract.node_id), Error::<T>::NodeNotExists);

		if contract.public_ips > 0 {
			Self::_reserve_ip(contract.node_id.clone(), &contract.public_ips, id)?
		}

        Contracts::<T>::insert(id, &contract);
        ContractID::put(id);

        Self::deposit_event(RawEvent::ContractCreated(id, contract.twin_id, contract.workload, contract.public_ips, address));

        Ok(())
	}

	pub fn _cancel_contract(address: T::AccountId, contract_id: u128) -> DispatchResult {
		ensure!(Contracts::<T>::contains_key(contract_id), Error::<T>::ContractNotExists);

		let contract = Contracts::<T>::get(contract_id);
		let twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id);
		debug::info!("twin address {:?}, signee {:?}", twin.address, address);
		ensure!(twin.address == address, Error::<T>::TwinNotAuthorizedToCancelContract);

		if contract.public_ips > 0 {
			Self::_free_ip(contract.node_id, contract_id)?
		}

        Contracts::<T>::remove(contract_id);

        Self::deposit_event(RawEvent::ContractCanceled(contract_id));

        Ok(())
	}

	pub fn _deploy_contract(address: T::AccountId, contract_id: u128) -> DispatchResult {
		ensure!(Contracts::<T>::contains_key(contract_id), Error::<T>::ContractNotExists);

		let mut contract = Contracts::<T>::get(contract_id);
		let node_id = pallet_tfgrid::NodesByPubkeyID::<T>::get(&contract.node_id);
		let node = pallet_tfgrid::Nodes::<T>::get(node_id);

		ensure!(node.address == address, Error::<T>::NodeNotAuthorizedToDeployContract);

		contract.state = ContractState::Deployed;
        Contracts::<T>::insert(contract_id, &contract);

		Self::deposit_event(RawEvent::ContractDeployed(contract_id, address));

		Ok(())
	}

	pub fn _reserve_ip(node_id: T::AccountId, number_of_ips_to_reserve: &u32, contract_id: u128) -> DispatchResult {
		let node_id = pallet_tfgrid::NodesByPubkeyID::<T>::get(&node_id);
		let node = pallet_tfgrid::Nodes::<T>::get(node_id);

		ensure!(pallet_tfgrid::Farms::contains_key(&node.farm_id), Error::<T>::FarmNotExists);
		let mut farm = pallet_tfgrid::Farms::get(node.farm_id);

		debug::info!("Number of farm ips {:?}, number of ips to reserve: {:?}", farm.public_ips.len(), *number_of_ips_to_reserve as usize);
		ensure!(farm.public_ips.len() >= *number_of_ips_to_reserve as usize, Error::<T>::FarmHasNotEnoughPublicIPs);

		let mut ips = Vec::new();
		for i in 0..farm.public_ips.len() {
			let mut ip = farm.public_ips[i].clone();

			if ips.len() == *number_of_ips_to_reserve as usize {
				break;
			}

			// if an ip has contract id 0 it means it's not reserved
			// reserve it now
			if ip.contract_id == 0 {
				ip.contract_id = contract_id;
				farm.public_ips[i] = ip.clone();
				ips.push(ip.ip);
			}
		}

		// Safeguard check if we actually have the amount of ips we wanted to reserve
		ensure!(ips.len() == *number_of_ips_to_reserve as usize, Error::<T>::FarmHasNotEnoughPublicIPsFree);

		// Update the farm with the reserved ips
		pallet_tfgrid::Farms::insert(farm.id, farm);

		// Emit an event containing the IP's reserved for this contract
		Self::deposit_event(RawEvent::IPsReserved(contract_id, ips));

		Ok(())
	}

	pub fn _free_ip(node_id: T::AccountId, contract_id: u128)  -> DispatchResult {
		let node_id = pallet_tfgrid::NodesByPubkeyID::<T>::get(&node_id);
		let node = pallet_tfgrid::Nodes::<T>::get(node_id);

		ensure!(pallet_tfgrid::Farms::contains_key(&node.farm_id), Error::<T>::FarmNotExists);
		let mut farm = pallet_tfgrid::Farms::get(node.farm_id);

		let mut ips_freed = Vec::new();
		for i in 0..farm.public_ips.len() {
			let mut ip = farm.public_ips[i].clone();

			// if an ip has contract id 0 it means it's not reserved
			// reserve it now
			if ip.contract_id == contract_id {
				ip.contract_id = 0;
				farm.public_ips[i] = ip.clone();
				ips_freed.push(ip.ip);
			}
		}

		pallet_tfgrid::Farms::insert(farm.id, farm);

		// Emit an event containing the IP's freed for this contract
		Self::deposit_event(RawEvent::IPsFreed(contract_id, ips_freed));

		Ok(())
	}
}
