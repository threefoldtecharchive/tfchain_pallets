#![cfg_attr(not(feature = "std"), no_std)]

//! A Pallet to demonstrate using currency imbalances
//!
//! WARNING: never use this code in production (for demonstration/teaching purposes only)
//! it only checks for signed extrinsics to enable arbitrary minting/slashing!!!

use frame_support::{
	decl_event, decl_module,
	traits::{Currency, OnUnbalanced, ReservableCurrency},
};
use frame_system::{self as system, ensure_signed};

// balance type using reservable currency type
type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
	<<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::NegativeImbalance;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

	/// Currency type for this pallet.
	type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

	/// Handler for the unbalanced decrement when slashing (burning collateral)
	type Burn: OnUnbalanced<NegativeImbalanceOf<Self>>;
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
		Balance = BalanceOf<T>,
		BlockNumber = <T as system::Trait>::BlockNumber,
	{
		AccountDrained(AccountId, Balance, BlockNumber),
		AccountFunded(AccountId, Balance, BlockNumber),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		#[weight = 10_000]
		fn swap_from_stellar(origin, target: T::AccountId, amount: BalanceOf<T>){
            let _ = ensure_signed(origin)?;
            Self::mint_tft(target, amount);
        }

        #[weight = 10_000]
		fn swap_to_stellar(origin, target: T::AccountId, amount: BalanceOf<T>){
            let _ = ensure_signed(origin)?;
            Self::burn_tft(target, amount);
        }
	}
}

impl<T: Trait> Module<T> {
	pub fn mint_tft(target: T::AccountId, amount: BalanceOf<T>) {        
        T::Currency::deposit_creating(&target, amount);
    
        let now = <system::Module<T>>::block_number();
        Self::deposit_event(RawEvent::AccountFunded(target, amount, now));
    }

    pub fn burn_tft(target: T::AccountId, amount: BalanceOf<T>) {
        let imbalance = T::Currency::slash(&target, amount).0;
        T::Burn::on_unbalanced(imbalance);
    
        let now = <system::Module<T>>::block_number();
        Self::deposit_event(RawEvent::AccountDrained(target, amount, now));
    }
}
