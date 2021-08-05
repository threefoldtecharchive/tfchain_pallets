use codec::{Decode, Encode};

use pallet_tfgrid::types;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct NodeContract {
	pub version: u32,
	pub contract_id: u64,
	pub twin_id: u32,
	pub node_id: u32,
	// deployment_data is the encrypted deployment body. This encrypted the deployment with the **USER** public key. 
	// So only the user can read this data later on (or any other key that he keeps safe).
    // this data part is read only by the user and can actually hold any information to help him reconstruct his deployment or can be left empty.
	pub deployment_data: Vec<u8>,
	// Hash of the deployment, set by the user
	pub deployment_hash: Vec<u8>,
    pub public_ips: u32,
	pub state: ContractState,
	pub public_ips_list: Vec<types::PublicIP>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct ContractBillingInformation {
	pub previous_nu_reported: u64,
	pub last_updated: u64,
	pub amount_unbilled: u64,
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
	pub contract_id: u64,
	pub timestamp: u64,
	pub cru: u64,
	pub sru: u64,
	pub hru: u64,
	pub mru: u64,
	pub nru: u64
}