#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_event, decl_module, decl_storage, decl_error, ensure, debug,
	traits::{Vec},
	traits::{Currency, ExistenceRequirement::AllowDeath},
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::{
	DispatchResult, DispatchError,
	traits::SaturatedConversion,
};
use codec::{Decode, Encode};
use pallet_tfgrid;
use pallet_timestamp as timestamp;

use substrate_fixed::types::{U64F64};

pub trait Config: system::Config + pallet_tfgrid::Config + pallet_timestamp::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Currency: Currency<Self::AccountId>;
}

pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;

decl_event!(
	pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
    {
		ContractCreated(u64, u32, Vec<u8>, u32, AccountId),
		IPsReserved(u64, Vec<Vec<u8>>),
		ContractCanceled(u64),
		IPsFreed(u64, Vec<Vec<u8>>),
		ContractDeployed(u64, AccountId),
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
		NodeNotAuthorizedToComputeReport,
		PricingPolicyNotExists
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Contract<AccountId> {
	twin_id: u32,
	node_id: AccountId,
    workload: Vec<u8>,
    public_ips: u32,
	state: ContractState,
	last_updated: u64,
	previous_nu_reported: u64,
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Consumption {
	contract_id: u64,
	cru: u64,
	sru: u64,
	hru: u64,
	mru: u64,
	nru: u64
}

decl_storage! {
	trait Store for Module<T: Config> as c {
        pub Contracts get(fn contracts): map hasher(blake2_128_concat) u64 => Contract<T::AccountId>;
        ContractID: u64;
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
		fn cancel_contract(origin, contract_id: u64){
            let address = ensure_signed(origin)?;
            Self::_cancel_contract(address, contract_id)?;
		}

		#[weight = 10_000]
		fn deploy_contract(origin, contract_id: u64) {
			let address = ensure_signed(origin)?;
			Self::_deploy_contract(address, contract_id)?;
		}

		#[weight = 10_000]
		fn add_reports(origin, reports: Vec<Consumption>) {
			let address = ensure_signed(origin)?;
			Self::_compute_reports(address, reports)?;
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

	pub fn _cancel_contract(address: T::AccountId, contract_id: u64) -> DispatchResult {
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

	pub fn _deploy_contract(address: T::AccountId, contract_id: u64) -> DispatchResult {
		ensure!(Contracts::<T>::contains_key(contract_id), Error::<T>::ContractNotExists);

		let mut contract = Contracts::<T>::get(contract_id);
		let node_id = pallet_tfgrid::NodesByPubkeyID::<T>::get(&contract.node_id);
		let node = pallet_tfgrid::Nodes::<T>::get(node_id);

		ensure!(node.address == address, Error::<T>::NodeNotAuthorizedToDeployContract);

		contract.state = ContractState::Deployed;
		contract.last_updated = <timestamp::Module<T>>::get().saturated_into::<u64>() / 1000;
        Contracts::<T>::insert(contract_id, &contract);

		Self::deposit_event(RawEvent::ContractDeployed(contract_id, address));

		Ok(())
	}

	pub fn _compute_reports(source: T::AccountId, reports: Vec<Consumption>) -> DispatchResult {
		debug::info!("computing reports: {:?}", reports);

		for report in reports {
			ensure!(Contracts::<T>::contains_key(report.contract_id), Error::<T>::ContractNotExists);
			let mut contract = Contracts::<T>::get(report.contract_id);
			ensure!(contract.node_id == source, Error::<T>::NodeNotAuthorizedToComputeReport);
			
			let node_id = pallet_tfgrid::NodesByPubkeyID::<T>::get(&contract.node_id);
			let node = pallet_tfgrid::Nodes::<T>::get(node_id);
			ensure!(pallet_tfgrid::Farms::contains_key(&node.farm_id), Error::<T>::FarmNotExists);
			let farm = pallet_tfgrid::Farms::get(node.farm_id);

			ensure!(pallet_tfgrid::PricingPolicies::contains_key(farm.pricing_policy_id), Error::<T>::PricingPolicyNotExists);
			let pricing_policy = pallet_tfgrid::PricingPolicies::get(farm.pricing_policy_id);

			let now = <timestamp::Module<T>>::get().saturated_into::<u64>() / 1000;
			let seconds_elapsed = now - contract.last_updated;
			debug::info!("seconds elapsed: {:?}", seconds_elapsed);

			let su_used = U64F64::from_num(report.hru) / 1200 + U64F64::from_num(report.sru) / 300;
			let su_cost = U64F64::from_num(pricing_policy.su) * U64F64::from_num(seconds_elapsed) * su_used;
			debug::info!("su cost: {:?}", su_cost);

			let mru_used = U64F64::from_num(report.mru) / 4;
			let cru_used = U64F64::from_num(report.cru) / 2;
			let min = if mru_used < cru_used {
				mru_used
			} else {
				cru_used
			};
			let cu_cost = U64F64::from_num(pricing_policy.cu) * U64F64::from_num(seconds_elapsed) * min;
			debug::info!("cu cost: {:?}", cu_cost);

			let mut nu_cost = 0;
			let mut used_nru = report.nru;
			if used_nru > contract.previous_nu_reported {
				// calculate used nru by subtracting previous reported units minus what is reported now
				// this is because nru is in a counter that increases only
				used_nru -= contract.previous_nu_reported;
				// calculate the cost for nru based on the used nru
				nu_cost = pricing_policy.nu as u64 * seconds_elapsed * used_nru;
			}
			debug::info!("nu cost: {:?}", nu_cost);

			// save total
			let total = su_cost.ceil().to_num::<u64>() + cu_cost.ceil().to_num::<u64>() + nu_cost;
			debug::info!("total cost: {:?}", total);

			// get the contracts free balance
			let twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id);
			let balance: BalanceOf<T> = T::Currency::free_balance(&twin.address);
			debug::info!("free balance: {:?}", balance);
			
			let mut decomission = false;
			let balances_as_u128: u128 = balance.saturated_into::<u128>();
			// if the total amount due exceeds to the balance decomission contract
			// but first drain the account
			if total as u128 >= balances_as_u128 {
				decomission = true;				
			}

			// convert amount due to balance object
			let amount_due: BalanceOf<T> = BalanceOf::<T>::saturated_from(total);
			debug::info!("amount due: {:?}", amount_due);

			// fetch farmer twin
			let farmer_twin = pallet_tfgrid::Twins::<T>::get(farm.twin_id);
			debug::info!("Transfering: {:?} from contract {:?} to farmer {:?}", &amount_due, &twin.address, &farmer_twin.address);
            // Transfer currency to the farmers account
            T::Currency::transfer(&twin.address, &farmer_twin.address, amount_due, AllowDeath)
                .map_err(|_| DispatchError::Other("Can't make transfer"))?;

			if decomission {
				if contract.public_ips > 0 {
					Self::_free_ip(node.address, report.contract_id)?;
				}
				Contracts::<T>::remove(report.contract_id);
			} else {
				// update contract
				contract.last_updated = now;
				contract.previous_nu_reported = report.nru;
				Contracts::<T>::insert(report.contract_id, &contract);
			}
		}

		Ok(())
	}

	pub fn _reserve_ip(node_id: T::AccountId, number_of_ips_to_reserve: &u32, contract_id: u64) -> DispatchResult {
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

	pub fn _free_ip(node_id: T::AccountId, contract_id: u64)  -> DispatchResult {
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
