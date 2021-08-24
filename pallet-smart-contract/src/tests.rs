use crate::{mock::*, Error, RawEvent};
use frame_support::{assert_noop, assert_ok, traits::{OnFinalize, OnInitialize}};
use sp_runtime::{
	traits::SaturatedConversion,
};
use frame_system::{RawOrigin};

use pallet_tfgrid::types as pallet_tfgrid_types;
use super::types;

#[test]
fn test_create_contract_works() {
	new_test_ext().execute_with(|| {
		prepare_farm_and_node();

		assert_ok!(SmartContractModule::create_contract(Origin::signed(alice()), 1, "some_data".as_bytes().to_vec(), "hash".as_bytes().to_vec(), 0));
	});
}

#[test]
fn test_create_contract_with_undefined_node_fails() {
	new_test_ext().execute_with(|| {
		prepare_farm_and_node();

		assert_noop!(
			SmartContractModule::create_contract(Origin::signed(alice()), 2, "some_data".as_bytes().to_vec(), "hash".as_bytes().to_vec(), 0),
			Error::<TestRuntime>::NodeNotExists
		);
	});
}


#[test]
fn test_create_contract_with_same_hash_and_node_fails() {
	new_test_ext().execute_with(|| {
		prepare_farm_and_node();

		assert_ok!(SmartContractModule::create_contract(Origin::signed(alice()), 1, "some_data".as_bytes().to_vec(), "hash".as_bytes().to_vec(), 0));

		assert_noop!(
			SmartContractModule::create_contract(Origin::signed(alice()), 1, "some_data".as_bytes().to_vec(), "hash".as_bytes().to_vec(), 0),
			Error::<TestRuntime>::ContractIsNotUnique
		);
	});
}

#[test]
fn test_update_contract_works() {
	new_test_ext().execute_with(|| {
		prepare_farm_and_node();

		assert_ok!(SmartContractModule::create_contract(Origin::signed(alice()), 1, "some_data".as_bytes().to_vec(), "hash".as_bytes().to_vec(), 0));

		assert_ok!(SmartContractModule::update_contract(Origin::signed(alice()), 1, "no_data".as_bytes().to_vec(), "some_other_hash".as_bytes().to_vec()));

		let expected_contract_value = types::NodeContract {
			node_id: 1,
			contract_id: 1,
			deployment_data: "no_data".as_bytes().to_vec(),
			deployment_hash: "some_other_hash".as_bytes().to_vec(),
			public_ips: 0,
			public_ips_list: Vec::new(),
			state: types::ContractState::Created,
			twin_id: 1,
			version: 1
		};

		let node_contract = SmartContractModule::contracts(1);
		assert_eq!(node_contract, expected_contract_value);

		let contracts = SmartContractModule::node_contracts(1, types::ContractState::Created);
		assert_eq!(contracts.len(), 1);

		assert_eq!(contracts[0], expected_contract_value);

		let node_contract_id_by_hash = SmartContractModule::node_contract_by_hash(1, "some_other_hash".as_bytes().to_vec());
		assert_eq!(node_contract_id_by_hash, 1);
	});
}

#[test]
fn test_update_contract_not_exists_fails() {
	new_test_ext().execute_with(|| {
		prepare_farm_and_node();

		assert_noop!(
			SmartContractModule::update_contract(Origin::signed(alice()), 1, "some_data".as_bytes().to_vec(), "hash".as_bytes().to_vec()),
			Error::<TestRuntime>::ContractNotExists
		);
	});
}

#[test]
fn test_update_contract_wrong_twins_fails() {
	new_test_ext().execute_with(|| {
		prepare_farm_and_node();

		assert_ok!(SmartContractModule::create_contract(Origin::signed(alice()), 1, "some_data".as_bytes().to_vec(), "hash".as_bytes().to_vec(), 0));

		assert_noop!(
			SmartContractModule::update_contract(Origin::signed(bob()), 1, "some_data".as_bytes().to_vec(), "hash".as_bytes().to_vec()),
			Error::<TestRuntime>::TwinNotAuthorizedToUpdateContract
		);
	});
}


#[test]
fn test_cancel_contract_works() {
	new_test_ext().execute_with(|| {
		prepare_farm_and_node();

		assert_ok!(SmartContractModule::create_contract(Origin::signed(alice()), 1, "some_data".as_bytes().to_vec(), "hash".as_bytes().to_vec(), 0));

		assert_ok!(SmartContractModule::cancel_contract(Origin::signed(alice()), 1));

		let expected_contract_value = types::NodeContract {
			node_id: 1,
			contract_id: 1,
			deployment_data: "some_data".as_bytes().to_vec(),
			deployment_hash: "hash".as_bytes().to_vec(),
			public_ips: 0,
			public_ips_list: Vec::new(),
			state: types::ContractState::Deleted,
			twin_id: 1,
			version: 1
		};

		let node_contract = SmartContractModule::contracts(1);
		assert_eq!(node_contract, expected_contract_value);

		let contracts = SmartContractModule::node_contracts(1, types::ContractState::Created);
		assert_eq!(contracts.len(), 0);
	});
}

#[test]
fn test_cancel_contract_not_exists_fails() {
	new_test_ext().execute_with(|| {
		prepare_farm_and_node();

		assert_noop!(
			SmartContractModule::cancel_contract(Origin::signed(alice()), 1),
			Error::<TestRuntime>::ContractNotExists
		);
	});
}

#[test]
fn test_cancel_contract_wrong_twins_fails() {
	new_test_ext().execute_with(|| {
		prepare_farm_and_node();

		assert_ok!(SmartContractModule::create_contract(Origin::signed(alice()), 1, "some_data".as_bytes().to_vec(), "hash".as_bytes().to_vec(), 0));

		assert_noop!(
			SmartContractModule::cancel_contract(Origin::signed(bob()), 1),
			Error::<TestRuntime>::TwinNotAuthorizedToCancelContract
		);
	});
}

#[test]
fn test_simulate_billing() {
	new_test_ext().execute_with(|| {
		prepare_farm_and_node();
		run_to_block(1);

		Timestamp::set_timestamp(1628082000 * 1000);

		assert_ok!(SmartContractModule::create_contract(Origin::signed(bob()), 1, "some_data".as_bytes().to_vec(), "hash".as_bytes().to_vec(), 0));

		let contract_billing_info = SmartContractModule::contract_billing_information_by_id(1);
		assert_eq!(contract_billing_info.last_updated, 1628082000);

		let contract_to_bill = SmartContractModule::contract_to_bill_at_block(61);
		assert_eq!(contract_to_bill, [1]);
		
		let gigabyte = 1000*1000*1000;
		let mut consumption_reports = Vec::new();
		consumption_reports.push(super::types::Consumption{
			contract_id: 1,
			cru: 2,
			hru: 0,
			mru: 2*gigabyte,
			sru: 60*gigabyte,
			nru: 3*gigabyte,
			timestamp: 1628085600
		});

		let contract_billing_info = SmartContractModule::contract_billing_information_by_id(1);
		let seconds_elapsed = 1628085600 - contract_billing_info.last_updated;
		assert_eq!(seconds_elapsed, 3600);

		assert_ok!(SmartContractModule::add_reports(Origin::signed(alice()), consumption_reports));

		let contract_billing_info = SmartContractModule::contract_billing_information_by_id(1);
		assert_eq!(contract_billing_info.amount_unbilled, 3600017);

		// let mature 10 blocks
		// because we bill every 10 blocks
		run_to_block(62);

		// Test that the expected events were emitted
		let our_events = System::events()
		.into_iter()
		.map(|r| r.event)
		.filter_map(|e| {
			if let Event::pallet_smart_contract(inner) = e {
				Some(inner)
			} else {
				None
			}
		})
		.collect::<Vec<_>>();

		let contract_bill_event = types::ContractBill {
			contract_id: 1,
			timestamp: 1628082000,
			discount_level: types::DiscountLevel::None,
			amount_billed: 3600017
		};
		let expected_events = vec![
			RawEvent::ContractBilled(contract_bill_event),
		];

		assert_eq!(our_events[2], expected_events[0]);

		// check the farmer twins account and see if it got balanced debited
		let twin = TfgridModule::twins(1);
		let b = Balances::free_balance(&twin.account_id);
		let balances_as_u128: u128 = b.saturated_into::<u128>();
		assert_eq!(balances_as_u128, 1000002520012);

		// check the contract owners address to see if it got balance credited
		let twin = TfgridModule::twins(2);
		let b = Balances::free_balance(&twin.account_id);
		let balances_as_u128: u128 = b.saturated_into::<u128>();
		assert_eq!(balances_as_u128, 2497479988);

		// amount unbilled should have been reset after a transfer between contract owner and farmer
		let contract_billing_info = SmartContractModule::contract_billing_information_by_id(1);
		assert_eq!(contract_billing_info.amount_unbilled, 0);
	});
}

#[test]
fn test_name_registration_works() {
	new_test_ext().execute_with(|| {
		prepare_farm_and_node();

		assert_ok!(SmartContractModule::register_name(Origin::signed(alice()), "foobar".as_bytes().to_vec()));
	});
}

#[test]
fn test_name_registration_fails_with_invalid_dns_name() {
	new_test_ext().execute_with(|| {
		prepare_farm_and_node();

		assert_noop!(
			SmartContractModule::register_name(Origin::signed(alice()), "foo.bar".as_bytes().to_vec()),
			Error::<TestRuntime>::NameNotValid
		);

		assert_noop!(
			SmartContractModule::register_name(Origin::signed(alice()), "foo!".as_bytes().to_vec()),
			Error::<TestRuntime>::NameNotValid
		);

		assert_noop!(
			SmartContractModule::register_name(Origin::signed(alice()), "foo;'".as_bytes().to_vec()),
			Error::<TestRuntime>::NameNotValid
		);

		assert_noop!(
			SmartContractModule::register_name(Origin::signed(alice()), "foo123.%".as_bytes().to_vec()),
			Error::<TestRuntime>::NameNotValid
		);
	});
}


fn prepare_farm_and_node() {
	let ip = "10.2.3.3";
	TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()).unwrap();

	let ip = "10.2.3.3";
	TfgridModule::create_twin(Origin::signed(bob()), ip.as_bytes().to_vec()).unwrap();

	let farm_name = "test_farm";
	let mut pub_ips = Vec::new();
	pub_ips.push(pallet_tfgrid_types::PublicIP{
		ip: "1.1.1.0".as_bytes().to_vec(),
		gateway: "1.1.1.1".as_bytes().to_vec(),
		contract_id: 0
	});

	let su_policy = pallet_tfgrid_types::Policy {
		value: 3000000,
		unit: pallet_tfgrid_types::Unit::Gigabytes,
	};
	let nu_policy = pallet_tfgrid_types::Policy {
		value: 20000,
		unit: pallet_tfgrid_types::Unit::Gigabytes,
	};
	let cu_policy = pallet_tfgrid_types::Policy {
		value: 6000000,
		unit: pallet_tfgrid_types::Unit::Gigabytes,
	};
	let ipu_policy = pallet_tfgrid_types::Policy::default();

	TfgridModule::create_pricing_policy(
		RawOrigin::Root.into(),
		"policy_1".as_bytes().to_vec(),
		su_policy,
		cu_policy,
		nu_policy,
		ipu_policy,
		bob(),
		bob()
	).unwrap();

	TfgridModule::create_farm(Origin::signed(alice()), farm_name.as_bytes().to_vec(), pub_ips.clone()).unwrap();

	// random location
	let location = pallet_tfgrid_types::Location{
		longitude: "12.233213231".as_bytes().to_vec(),
		latitude: "32.323112123".as_bytes().to_vec()
	};

	let resources = pallet_tfgrid_types::Resources {
		hru: 1,
		sru: 1,
		cru: 1,
		mru: 1,
	};

	let country = "Belgium".as_bytes().to_vec();
	let city = "Ghent".as_bytes().to_vec();
	TfgridModule::create_node(Origin::signed(alice()), 1, resources, location, country, city, None).unwrap();
}

fn run_to_block(n: u64) {
	while System::block_number() < n {
		SmartContractModule::on_finalize(System::block_number());
		System::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		SmartContractModule::on_initialize(System::block_number());
	}
}