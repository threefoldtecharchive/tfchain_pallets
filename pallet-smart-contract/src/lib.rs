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
use pallet_tfgrid::types;

use substrate_fixed::types::{U64F64};

#[cfg(test)]
mod tests;

pub trait Config: system::Config + pallet_tfgrid::Config + pallet_timestamp::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Currency: Currency<Self::AccountId>;
}

pub const CONTRACT_VERSION: u32 = 1;

pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;

decl_event!(
	pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
    {
		ContractCreated(NodeContract),
		ContractUpdated(NodeContract),
		IPsReserved(u64, Vec<types::PublicIP>),
		ContractCanceled(u64),
		IPsFreed(u64, Vec<Vec<u8>>),
		ContractDeployed(u64, AccountId),
		ConsumptionReportReceived(Consumption),
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
		PricingPolicyNotExists,
		ContractIsNotUnique
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct NodeContract {
	version: u32,
	contract_id: u64,
	twin_id: u32,
	node_id: u32,
	// deployment_data is the encrypted deployment body. This encrypted the deployment with the **USER** public key. 
	// So only the user can read this data later on (or any other key that he keeps safe).
    // this data part is read only by the user and can actually hold any information to help him reconstruct his deployment or can be left empty.
	deployment_data: Vec<u8>,
	// Hash of the deployment, set by the user
	deployment_hash: Vec<u8>,
    public_ips: u32,
	state: ContractState,
	public_ips_list: Vec<types::PublicIP>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct ContractBillingInformation {
	previous_nu_reported: u64,
	last_updated: u64,
	amount_unbilled: u64,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
pub enum ContractState {
	Created,
	Deleted,
	OutOfFunds,
}

impl Default for ContractState {
	fn default() -> ContractState {
		ContractState::Created
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Consumption {
	contract_id: u64,
	timestamp: u64,
	cru: u64,
	sru: u64,
	hru: u64,
	mru: u64,
	nru: u64
}

decl_storage! {
	trait Store for Module<T: Config> as SmartContractModule {
        pub Contracts get(fn contracts): map hasher(blake2_128_concat) u64 => NodeContract;
		pub ContractBillingInformationByID get(fn contract_billing_information_by_id): map hasher(blake2_128_concat) u64 => ContractBillingInformation;
		// ContractIDByNodeIDAndHash is a mapping for a contract ID by supplying a node_id and a deployment_hash
		// this combination makes a deployment for a user / node unique
		pub ContractIDByNodeIDAndHash get(fn node_contract_by_hash): double_map hasher(blake2_128_concat) u32, hasher(blake2_128_concat) Vec<u8> => u64;
		pub NodeContracts get(fn node_contracts): double_map hasher(blake2_128_concat) u32, hasher(blake2_128_concat) ContractState => Vec<NodeContract>;
		pub ContractsToBillAt get(fn ip_expires_at): map hasher(blake2_128_concat) u64 => Vec<u64>;
        ContractID: u64;
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		fn deposit_event() = default;
		
		#[weight = 10]
		fn create_contract(origin, node_id: u32, data: Vec<u8>, deployment_hash: Vec<u8>, public_ips: u32){
            let account_id = ensure_signed(origin)?;
            Self::_create_contract(account_id, node_id, data, deployment_hash, public_ips)?;
		}

		#[weight = 10]
		fn update_contract(origin, contract_id: u64, data: Vec<u8>, deployment_hash: Vec<u8>){
            let account_id = ensure_signed(origin)?;
            Self::_update_contract(account_id, contract_id, data, deployment_hash)?;
		}

		#[weight = 10]
		fn cancel_contract(origin, contract_id: u64){
            let account_id = ensure_signed(origin)?;
            Self::_cancel_contract(account_id, contract_id)?;
		}

		#[weight = 10]
		fn add_reports(origin, reports: Vec<Consumption>) {
			let account_id = ensure_signed(origin)?;
			Self::_compute_reports(account_id, reports)?;
		}

		fn on_finalize(block: T::BlockNumber) {
			debug::info!("Entering on finalize: {:?}", block);
			match Self::_bill_contracts_at_block(block) {
				Ok(_) => {
					debug::info!("NodeContract billed successfully at block: {:?}", block);
				},
				Err(err) => {
					debug::info!("NodeContract billed failed at block: {:?} with err {:?}", block, err);
				}
			}
		}
	}
}

impl<T: Config> Module<T> {
	pub fn _create_contract(account_id: T::AccountId, node_id: u32, deployment_data: Vec<u8>, deployment_hash: Vec<u8>, public_ips: u32) -> DispatchResult {
		let mut id = ContractID::get();
		id = id+1;
		
		ensure!(!ContractIDByNodeIDAndHash::contains_key(node_id, &deployment_hash), Error::<T>::ContractIsNotUnique);
		ensure!(pallet_tfgrid::TwinIdByAccountID::<T>::contains_key(&account_id), Error::<T>::TwinNotExists);
		let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&account_id);
		let twin = pallet_tfgrid::Twins::<T>::get(twin_id);
		ensure!(twin.account_id == account_id, Error::<T>::TwinNotAuthorizedToCreateContract);

		ensure!(pallet_tfgrid::Nodes::contains_key(&node_id), Error::<T>::NodeNotExists);

		let mut contract = NodeContract {
			version: CONTRACT_VERSION,
			contract_id: id,
			node_id,
			deployment_data,
			deployment_hash: deployment_hash.clone(),
			public_ips,
			twin_id,
			state: ContractState::Created,
			public_ips_list: Vec::new()
		};

		let contract_billing_information = ContractBillingInformation {
			last_updated: <timestamp::Module<T>>::get().saturated_into::<u64>() / 1000,
			amount_unbilled: 0,
			previous_nu_reported: 0
		};

		Self::_reserve_ip(&mut contract)?;

		Self::_reinsert_contract_to_bill(contract.contract_id)?;

        Contracts::insert(id, &contract);
        ContractID::put(id);
		ContractBillingInformationByID::insert(id, contract_billing_information);
		ContractIDByNodeIDAndHash::insert(node_id, deployment_hash, id);
		
		let mut node_contracts = NodeContracts::get(&contract.node_id, &contract.state);
		node_contracts.push(contract.clone());
		NodeContracts::insert(&contract.node_id, &contract.state, &node_contracts);

        Self::deposit_event(RawEvent::ContractCreated(contract));

        Ok(())
	}

	pub fn _update_contract(account_id: T::AccountId, contract_id: u64, deployment_data: Vec<u8>, deployment_hash: Vec<u8>) -> DispatchResult {
		ensure!(Contracts::contains_key(contract_id), Error::<T>::ContractNotExists);

		let mut contract = Contracts::get(contract_id);
		let twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id);
		ensure!(twin.account_id == account_id, Error::<T>::TwinNotAuthorizedToCancelContract);

		contract.deployment_data = deployment_data;
		contract.deployment_hash = deployment_hash;
		Contracts::insert(contract_id, &contract);

		Self::deposit_event(RawEvent::ContractUpdated(contract));

		Ok(())
	}

	pub fn _cancel_contract(account_id: T::AccountId, contract_id: u64) -> DispatchResult {
		ensure!(Contracts::contains_key(contract_id), Error::<T>::ContractNotExists);

		let mut contract = Contracts::get(contract_id);
		let twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id);
		ensure!(twin.account_id == account_id, Error::<T>::TwinNotAuthorizedToCancelContract);

		if contract.public_ips > 0 {
			Self::_free_ip(&mut contract)?
		}

		// remove the contract by hash from storage
		ContractIDByNodeIDAndHash::remove(contract.node_id, &contract.deployment_hash);

		Self::_update_contract_state(contract, ContractState::Deleted)?;

        Self::deposit_event(RawEvent::ContractCanceled(contract_id));

        Ok(())
	}

	pub fn _compute_reports(source: T::AccountId, reports: Vec<Consumption>) -> DispatchResult {
		ensure!(pallet_tfgrid::TwinIdByAccountID::<T>::contains_key(&source), Error::<T>::TwinNotExists);
		let twin_id = pallet_tfgrid::TwinIdByAccountID::<T>::get(&source);
		ensure!(pallet_tfgrid::NodeIdByTwinID::contains_key(twin_id), Error::<T>::NodeNotExists);

		// fetch the node from the source account (signee)
		let node_id = pallet_tfgrid::NodeIdByTwinID::get(&twin_id);
		let node = pallet_tfgrid::Nodes::get(node_id);
		
		ensure!(pallet_tfgrid::Farms::contains_key(&node.farm_id), Error::<T>::FarmNotExists);
		let farm = pallet_tfgrid::Farms::get(node.farm_id);
		
		ensure!(pallet_tfgrid::PricingPolicies::contains_key(farm.pricing_policy_id), Error::<T>::PricingPolicyNotExists);
		let pricing_policy = pallet_tfgrid::PricingPolicies::get(farm.pricing_policy_id);

		// validation
		for report in &reports {
		  if !Contracts::contains_key(report.contract_id) {
			continue;
		  }
		  let contract = Contracts::get(report.contract_id);
		  ensure!(contract.node_id == twin_id, Error::<T>::NodeNotAuthorizedToComputeReport);
		  ensure!(ContractBillingInformationByID::contains_key(report.contract_id), Error::<T>::ContractNotExists);
		}

		for report in reports {
			if !ContractBillingInformationByID::contains_key(report.contract_id) {
				continue;
			}

			let mut contract_billing_info = ContractBillingInformationByID::get(report.contract_id);
			if report.timestamp < contract_billing_info.last_updated {
				continue;
			}

			let seconds_elapsed = report.timestamp - contract_billing_info.last_updated;
			debug::info!("seconds elapsed: {:?}", seconds_elapsed);

			
			let factor = match pricing_policy.unit {
				pallet_tfgrid::types::Unit::Bytes => {
					1
				}
				pallet_tfgrid::types::Unit::Kilobytes => {
					1024
				}
				pallet_tfgrid::types::Unit::Megabytes => {
					1024 * 1024
				}
				pallet_tfgrid::types::Unit::Gigabytes => {
					1024 * 1024 * 1024
				}
			};

			let hru = U64F64::from_num(report.hru) / factor;
			let sru = U64F64::from_num(report.sru) / factor;
			let mru = U64F64::from_num(report.mru) / factor;

			let su_used = hru / 1200 + sru / 300;
			let su_cost = U64F64::from_num(pricing_policy.su) * U64F64::from_num(seconds_elapsed) * su_used;
			debug::info!("su cost: {:?}", su_cost);

			let mru_used = mru / 4;
			let cru_used = U64F64::from_num(report.cru) / 2;
			let min = if mru_used < cru_used {
				mru_used
			} else {
				cru_used
			};
			let cu_cost = U64F64::from_num(pricing_policy.cu) * U64F64::from_num(seconds_elapsed) * min;
			debug::info!("cu cost: {:?}", cu_cost);

			let mut used_nru = U64F64::from_num(report.nru) / factor;
			let nu_cost = if used_nru > contract_billing_info.previous_nu_reported {
				// calculate used nru by subtracting previous reported units minus what is reported now
				// this is because nru is in a counter that increases only
				used_nru -= U64F64::from_num(contract_billing_info.previous_nu_reported);

				// calculate the cost for nru based on the used nru
				used_nru * U64F64::from_num(pricing_policy.nu)
			} else {
				U64F64::from_num(0)
			};

			debug::info!("nu cost: {:?}", nu_cost);

			// save total
			let total = su_cost + cu_cost + nu_cost;
			let total = total.ceil().to_num::<u64>();
			debug::info!("total cost: {:?}", total);

			// update contract_billing_info
			contract_billing_info.amount_unbilled += total;
			contract_billing_info.last_updated = report.timestamp;
			ContractBillingInformationByID::insert(report.contract_id, &contract_billing_info);
			Self::deposit_event(RawEvent::ConsumptionReportReceived(report));
		}

		Ok(())
	}

	pub fn _bill_contracts_at_block(block: T::BlockNumber) -> DispatchResult {
		let current_block_u64: u64 = block.saturated_into::<u64>();
		let contracts = ContractsToBillAt::get(current_block_u64);

		debug::info!("Contracts to check at block: {:?}, {:?}", block, contracts);
		if contracts.len() == 0 {
			return Ok(())
		}

		for contract_id in contracts {
			let mut contract = Contracts::get(contract_id);
			if contract.state != ContractState::Created {
				continue
			}
			let mut contract_billing_info = ContractBillingInformationByID::get(contract_id);
			
			let node = pallet_tfgrid::Nodes::get(contract.node_id);
			
			ensure!(pallet_tfgrid::Farms::contains_key(&node.farm_id), Error::<T>::FarmNotExists);
			let farm = pallet_tfgrid::Farms::get(node.farm_id);
			
			ensure!(pallet_tfgrid::PricingPolicies::contains_key(farm.pricing_policy_id), Error::<T>::PricingPolicyNotExists);
			let pricing_policy = pallet_tfgrid::PricingPolicies::get(farm.pricing_policy_id);
			
			// bill user for 1 hour ip usage (10 blocks * 6 seconds)
			let total_ip_cost = contract.public_ips * pricing_policy.ipu * (10 * 6);

			let total = total_ip_cost as u64 + contract_billing_info.amount_unbilled;

			if total == 0 {
				Self::_reinsert_contract_to_bill(contract.contract_id)?;
				continue
			}

			let amount_due: BalanceOf<T> = BalanceOf::<T>::saturated_from(total);

			// get the contracts free balance
			let twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id);
			let balance: BalanceOf<T> = T::Currency::free_balance(&twin.account_id);
			debug::info!("free balance: {:?}", balance);
			
			let mut decomission = false;
			let balances_as_u128: u128 = balance.saturated_into::<u128>();
			// if the total amount due exceeds to the balance decomission contract
			// but first drain the account
			if total as u128 >= balances_as_u128 {
				decomission = true;				
			}

			let amount_due_as_u128: u128 = amount_due.saturated_into::<u128>();
			// calculate amount due on a monthly basis
			// we bill every one minute so we do 60 * 24 * 12
			let amount_due_monthly = amount_due_as_u128 * 60 * 24 * 12;
			// see how many months a user can pay for this deployment given his balance
			let discount_level = U64F64::from_num(balances_as_u128) / U64F64::from_num(amount_due_monthly);
    
			// predefined discount levels
			// https://wiki.threefold.io/#/threefold__grid_pricing
			let discount = match discount_level.ceil().to_num::<u64>() {
				d if d >= 3 && d < 6 => U64F64::from_num(0.2),
				d if d >= 6 && d < 12 => U64F64::from_num(0.3),
				d if d >= 12 && d < 36 => U64F64::from_num(0.4),
				d if d >= 36 => U64F64::from_num(0.6),
				_ => U64F64::from_num(1),
			};
			
			// calculate the new amount due given the discount
			let amount_due = U64F64::from_num(amount_due_as_u128) * discount;
			// convert to balance object
			let amount_due: BalanceOf<T> = BalanceOf::<T>::saturated_from(amount_due.ceil().to_num::<u64>());

			// fetch source twin
			let twin = pallet_tfgrid::Twins::<T>::get(contract.twin_id);
			// fetch farmer twin
			let farmer_twin = pallet_tfgrid::Twins::<T>::get(farm.twin_id);
			debug::info!("Transfering: {:?} from contract {:?} to farmer {:?}", &amount_due, &twin.account_id, &farmer_twin.account_id);
			// Transfer currency to the farmers account
			T::Currency::transfer(&twin.account_id, &farmer_twin.account_id, amount_due, AllowDeath)
				.map_err(|_| DispatchError::Other("Can't make transfer"))?;

			if decomission {
				if contract.public_ips > 0 {
					Self::_free_ip(&mut contract)?;
				}
				Self::_update_contract_state(contract, ContractState::OutOfFunds)?;
				continue
			}

			// set the amount unbilled back to 0
			contract_billing_info.amount_unbilled = 0;
			ContractBillingInformationByID::insert(contract.contract_id, &contract_billing_info);

			Self::_reinsert_contract_to_bill(contract.contract_id)?;
		}
		Ok(())
	}

	pub fn _reinsert_contract_to_bill(contract_id: u64) -> DispatchResult {
		// Save the contract
		let now = <frame_system::Module<T>>::block_number().saturated_into::<u64>();
		// Save the contract to be billed in 10 blocks
		let future_block = now + 10;
		let mut contracts = ContractsToBillAt::get(future_block);
		contracts.push(contract_id);
		ContractsToBillAt::insert(future_block, &contracts);
		debug::info!("Insert contracts: {:?}, to be billed at block {:?}", contracts, future_block);
		Ok(())
	}

	pub fn _update_contract_state(mut contract: NodeContract, state: ContractState) -> DispatchResult {
		// Remove contract from double map first
		let mut contracts = NodeContracts::get(&contract.node_id, &contract.state);

		match contracts.iter().position(|ct| ct.contract_id == contract.contract_id) {
			Some(index) => {
				contracts.remove(index);
				NodeContracts::insert(&contract.node_id, &contract.state, &contracts);
			},
			None => ()
		};

		// Assign new state
		contract.state = state;
		
		// Re-insert new values
		let mut contracts = NodeContracts::get(&contract.node_id, &contract.state);
		contracts.push(contract.clone());
		NodeContracts::insert(&contract.node_id, &contract.state, &contracts);
		
		Contracts::insert(&contract.contract_id, &contract);
		
		Ok(())
	}

	pub fn _reserve_ip(contract: &mut NodeContract) -> DispatchResult {
		if contract.public_ips == 0 {
			return Ok(());
		}
		let node = pallet_tfgrid::Nodes::get(contract.node_id);

		ensure!(pallet_tfgrid::Farms::contains_key(&node.farm_id), Error::<T>::FarmNotExists);
		let mut farm = pallet_tfgrid::Farms::get(node.farm_id);

		debug::info!("Number of farm ips {:?}, number of ips to reserve: {:?}", farm.public_ips.len(), contract.public_ips as usize);
		ensure!(farm.public_ips.len() >= contract.public_ips as usize, Error::<T>::FarmHasNotEnoughPublicIPs);

		let mut ips = Vec::new();
		for i in 0..farm.public_ips.len() {
			let mut ip = farm.public_ips[i].clone();

			if ips.len() == contract.public_ips as usize {
				break;
			}

			// if an ip has contract id 0 it means it's not reserved
			// reserve it now
			if ip.contract_id == 0 {
				ip.contract_id = contract.contract_id;
				farm.public_ips[i] = ip.clone();
				ips.push(ip);
			}
		}

		// Safeguard check if we actually have the amount of ips we wanted to reserve
		ensure!(ips.len() == contract.public_ips as usize, Error::<T>::FarmHasNotEnoughPublicIPsFree);

		// Update the farm with the reserved ips
		pallet_tfgrid::Farms::insert(farm.id, farm);

		contract.public_ips_list = ips;

		Ok(())
	}

	pub fn _free_ip(contract: &mut NodeContract)  -> DispatchResult {
		let node = pallet_tfgrid::Nodes::get(contract.node_id);

		ensure!(pallet_tfgrid::Farms::contains_key(&node.farm_id), Error::<T>::FarmNotExists);
		let mut farm = pallet_tfgrid::Farms::get(node.farm_id);

		let mut ips_freed = Vec::new();
		for i in 0..farm.public_ips.len() {
			let mut ip = farm.public_ips[i].clone();

			// if an ip has contract id 0 it means it's not reserved
			// reserve it now
			if ip.contract_id == contract.contract_id {
				ip.contract_id = 0;
				farm.public_ips[i] = ip.clone();
				ips_freed.push(ip.ip);
			}
		}

		pallet_tfgrid::Farms::insert(farm.id, farm);

		// Emit an event containing the IP's freed for this contract
		Self::deposit_event(RawEvent::IPsFreed(contract.contract_id, ips_freed));

		Ok(())
	}
}
