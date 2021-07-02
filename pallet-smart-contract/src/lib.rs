#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_event, decl_module, decl_storage, decl_error, ensure,
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
		ContractStored(u32, Vec<u8>, u32, AccountId),
	}
);

decl_error! {
	/// Error for the smart contract module.
	pub enum Error for Module<T: Config> {
		TwinNotExists,
		NodeNotExists,
		FarmNotExists
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct Contract<AccountId> {
	twin_id: u32,
	node_id: AccountId,
    workload: Vec<u8>,
    public_ips: u32
}

decl_storage! {
	trait Store for Module<T: Config> as VestingValidatorModule {
        pub Contracts get(fn contracts): map hasher(blake2_128_concat) u32 => Contract<T::AccountId>;
        ContractID: u32;
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

		fn on_finalize(block: T::BlockNumber) {
			
		}
	}
}

impl<T: Config> Module<T> {
	pub fn _create_contract(address: T::AccountId, contract: Contract<T::AccountId>) -> DispatchResult {
		let mut id = ContractID::get();
		id = id+1;
		
		ensure!(pallet_tfgrid::Twins::<T>::contains_key(&contract.twin_id), Error::<T>::TwinNotExists);
		ensure!(pallet_tfgrid::NodesByPubkeyID::<T>::contains_key(&contract.node_id), Error::<T>::NodeNotExists);

		let node_id = pallet_tfgrid::NodesByPubkeyID::<T>::get(&contract.node_id);
		let node = pallet_tfgrid::Nodes::<T>::get(node_id);
		
		ensure!(pallet_tfgrid::Farms::contains_key(&node.farm_id), Error::<T>::FarmNotExists);
		let farm = pallet_tfgrid::Farms::get(node.farm_id);

		let public_ips_reserved: Vec<u8> = Vec::new();

        Contracts::<T>::insert(id, &contract);
        ContractID::put(id);

        Self::deposit_event(RawEvent::ContractStored(contract.twin_id, contract.workload, contract.public_ips, address));

        Ok(())
	}
}
