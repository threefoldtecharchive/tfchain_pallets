use crate::{mock::*, Error};
use frame_support::{assert_ok, assert_noop};

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

		assert_ok!(SmartContractModule::update_contract(Origin::signed(alice()), 1, "some_data".as_bytes().to_vec(), "hash".as_bytes().to_vec()));
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

fn prepare_farm_and_node() {
	let ip = "10.2.3.3";
	TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()).unwrap();

	let ip = "10.2.3.3";
	TfgridModule::create_twin(Origin::signed(bob()), ip.as_bytes().to_vec()).unwrap();

	let farm_name = "test_farm";
	let mut pub_ips = Vec::new();
	pub_ips.push(super::types::PublicIP{
		ip: "1.1.1.0".as_bytes().to_vec(),
		gateway: "1.1.1.1".as_bytes().to_vec(),
		contract_id: 0
	});
	TfgridModule::create_farm(Origin::signed(alice()), farm_name.as_bytes().to_vec(), 0, super::types::CertificationType::Diy, 0, 0, pub_ips.clone()).unwrap();


	// random location
	let location = super::types::Location{
		longitude: "12.233213231".as_bytes().to_vec(),
		latitude: "32.323112123".as_bytes().to_vec()
	};

	let resources = super::types::Resources {
		hru: 1,
		sru: 1,
		cru: 1,
		mru: 1,
	};

	TfgridModule::create_node(Origin::signed(alice()), 1, resources, location, 0, 0, None).unwrap();
}