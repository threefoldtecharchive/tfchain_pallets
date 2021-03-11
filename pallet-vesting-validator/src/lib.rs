#![cfg_attr(not(feature = "std"), no_std)]

//! A Pallet to demonstrate using currency imbalances
//!
//! WARNING: never use this code in production (for demonstration/teaching purposes only)
//! it only checks for signed extrinsics to enable arbitrary minting/slashing!!!

use frame_support::{
	decl_event, decl_module, decl_storage, decl_error, ensure, debug,
	traits::{Currency, ReservableCurrency, Vec},
};
use frame_system::{self as system, ensure_signed, ensure_root};
use sp_runtime::{DispatchResult};
use codec::{Decode, Encode};
use sp_runtime::traits::SaturatedConversion;

// balance type using reservable currency type
type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;

pub trait Config: system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

	/// Currency type for this pallet.
	type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Config>::AccountId,
		Balance = BalanceOf<T>,
	{
		TransactionProposed(Vec<u8>, AccountId, Balance),
		TransactionSignatureAdded(Vec<u8>, Vec<u8>),
		TransactionReady(Vec<u8>),
		TransactionRemoved(Vec<u8>),
		TransactionExpired(Vec<u8>),
	}
);

decl_error! {
	/// Error for the vesting module.
	pub enum Error for Module<T: Config> {
		ValidatorExists,
		ValidatorNotExists,
		TransactionValidatorExists,
		TransactionValidatorNotExists,
		TransactionExists,
		TransactionNotExists,
		SignatureExists
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, Debug)]
pub struct StellarTransaction <BalanceOf, AccountId, BlockNumber>{
	pub amount: BalanceOf,
	pub target: AccountId,
	pub block: BlockNumber,
	pub signatures: Vec<Vec<u8>>
}

decl_storage! {
	trait Store for Module<T: Config> as TFTBridgeModule {
		pub Validators get(fn validator_accounts): Vec<T::AccountId>;

		pub Transactions get(fn transactions): map hasher(blake2_128_concat) Vec<u8> => StellarTransaction<BalanceOf<T>, T::AccountId, T::BlockNumber>;
		
		pub ExpiredTransactions get(fn expired_transactions): map hasher(blake2_128_concat) Vec<u8> => StellarTransaction<BalanceOf<T>, T::AccountId, T::BlockNumber>;
		pub ExecutedTransactions get(fn executed_transactions): map hasher(blake2_128_concat) Vec<u8> => StellarTransaction<BalanceOf<T>, T::AccountId, T::BlockNumber>;
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		fn deposit_event() = default;
		
		#[weight = 10_000]
		fn add_validator(origin, target: T::AccountId){
            ensure_root(origin)?;
            Self::add_validator_account(target)?;
		}
		
		#[weight = 10_000]
		fn remove_validator(origin, target: T::AccountId){
            ensure_root(origin)?;
            Self::remove_validator_account(target)?;
		}
		
		#[weight = 10_000]
		fn propose_transaction(origin, transaction: Vec<u8>, target: T::AccountId, amount: BalanceOf<T>){
            let validator = ensure_signed(origin)?;
            Self::propose_stellar_transaction(validator, transaction, target, amount)?;
		}

		#[weight = 10_000]
		fn add_sig_transaction(origin, transaction: Vec<u8>, signature: Vec<u8>){
            let validator = ensure_signed(origin)?;
            Self::add_sig_stellar_transaction(validator, transaction, signature)?;
		}

		#[weight = 10_000]
		fn remove_transaction(origin, transaction: Vec<u8>){
            let validator = ensure_signed(origin)?;
            Self::remove_stellar_transaction(validator, transaction)?;
		}

		fn on_finalize(block: T::BlockNumber) {
			let current_block_u64: u64 = block.saturated_into::<u64>();

			for (tx_id, tx) in Transactions::<T>::iter() {
				let tx_block_u64: u64 = tx.block.saturated_into::<u64>();
				// if 1000 blocks have passed since the tx got submitted
				// we can safely assume this tx is fault
				// add the faulty tx to the expired tx list
				if current_block_u64 - tx_block_u64 >= 1000 {
					// Remove tx from storage
					Transactions::<T>::remove(tx_id.clone());
					// Insert into expired transactions list
					ExpiredTransactions::<T>::insert(tx_id.clone(), tx);
					// Emit an expired event so validators can choose to retry
					Self::deposit_event(RawEvent::TransactionExpired(tx_id));
				}
			}
		}
	}
}

impl<T: Config> Module<T> {
	pub fn add_validator_account(target: T::AccountId) -> DispatchResult {
		let mut validators = Validators::<T>::get();

		match validators.binary_search(&target) {
			Ok(_) => Err(Error::<T>::ValidatorExists.into()),
			// If the search fails, the caller is not a member and we learned the index where
			// they should be inserted
			Err(index) => {
				validators.insert(index, target.clone());
				Validators::<T>::put(validators);
				Ok(())
			}
		}
	}

	pub fn remove_validator_account(target: T::AccountId) -> DispatchResult {
		let mut validators = Validators::<T>::get();

		match validators.binary_search(&target) {
			Ok(index) => {
				validators.remove(index);
				Validators::<T>::put(validators);
				Ok(())
			},
			Err(_) => Err(Error::<T>::ValidatorNotExists.into()),
		}
	}

	pub fn propose_stellar_transaction(origin: T::AccountId, tx_id: Vec<u8>, target: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
		// make sure we don't duplicate the transaction
		ensure!(!Transactions::<T>::contains_key(tx_id.clone()), Error::<T>::TransactionExists);
		
		let validators = Validators::<T>::get();
		match validators.binary_search(&origin) {
			Ok(_) => {
				let now = <frame_system::Module<T>>::block_number();
				let tx = StellarTransaction {
					amount,
					target: target.clone(),
					block: now,
					signatures: Vec::new()
				};
				Transactions::<T>::insert(tx_id.clone(), &tx);

				Self::deposit_event(RawEvent::TransactionProposed(tx_id, target, amount));

				Ok(())
			},
			Err(_) => Err(Error::<T>::ValidatorNotExists.into()),
		}
	}

	pub fn remove_stellar_transaction(origin: T::AccountId, tx_id: Vec<u8>) -> DispatchResult {
		// make sure we don't duplicate the transaction
		ensure!(!Transactions::<T>::contains_key(tx_id.clone()), Error::<T>::TransactionExists);
		
		let validators = Validators::<T>::get();
		match validators.binary_search(&origin) {
			Ok(_) => {
				let tx = Transactions::<T>::get(tx_id.clone());

				// Store it as an executed transaction
				ExecutedTransactions::<T>::insert(tx_id.clone(), &tx);

				// Remove it from the current transactions list
				Transactions::<T>::remove(tx_id.clone());

				Self::deposit_event(RawEvent::TransactionRemoved(tx_id));

				Ok(())
			},
			Err(_) => Err(Error::<T>::ValidatorNotExists.into()),
		}
	}

	pub fn add_sig_stellar_transaction(origin: T::AccountId, tx_id: Vec<u8>, signature: Vec<u8>) -> DispatchResult {
		// make sure tx exists
		ensure!(!Transactions::<T>::contains_key(tx_id.clone()), Error::<T>::TransactionExists);
		
		let validators = Validators::<T>::get();
		match validators.binary_search(&origin) {
			Ok(_) => {				
				let mut tx = Transactions::<T>::get(&tx_id.clone());
				
				// check if the signature already exists
				ensure!(!tx.signatures.iter().any(|c| c == &signature), Error::<T>::SignatureExists);

				// add the signature
				tx.signatures.push(signature.clone());

				// if more then then the half of all validators
				// submitted their signature we can emit an event that a transaction
				// is ready to be submitted to the stellar network
				if tx.signatures.len() > validators.len() / 2 {
					Self::deposit_event(RawEvent::TransactionReady(tx_id));
					return Ok(())
				}

 				Transactions::<T>::insert(tx_id.clone(), &tx);

				Self::deposit_event(RawEvent::TransactionSignatureAdded(tx_id, signature));

				Ok(())
			},
			Err(_) => Err(Error::<T>::ValidatorNotExists.into()),
		}
	}
}
