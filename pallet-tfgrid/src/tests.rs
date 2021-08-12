use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use frame_system::{RawOrigin};

#[test]
fn test_create_entity_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature));
	});
}

#[test]
fn test_update_entity_works() {
	ExternalityBuilder::build().execute_with(|| {
		let mut name = "foobar";

		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature));

		// Change name to barfoo
		name = "barfoo";

		assert_ok!(TfgridModule::update_entity(Origin::signed(test_ed25519()), name.as_bytes().to_vec(), 0,0));
	});
}

#[test]
fn test_update_entity_fails_if_signed_by_someone_else() {
	ExternalityBuilder::build().execute_with(|| {
		let mut name = "foobar";

		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature));
		
		// Change name to barfoo
		name = "barfoo";

		assert_noop!(
			TfgridModule::update_entity(Origin::signed(bob()), name.as_bytes().to_vec(), 0,0),
			Error::<TestRuntime>::EntityNotExists
		);
	});
}

#[test]
fn test_create_entity_double_fails() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature.clone()));

		assert_noop!(
			TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature),
			Error::<TestRuntime>::EntityWithNameExists
		);
	});
}

#[test]
fn test_create_entity_double_fails_with_same_pubkey() {
	ExternalityBuilder::build().execute_with(|| {
		let mut name = "foobar";

		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature.clone()));

		name = "barfoo";

		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_noop!(
			TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature),
			Error::<TestRuntime>::EntityWithPubkeyExists
		);
	});
}

#[test]
fn test_delete_entity_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature.clone()));

		assert_ok!(TfgridModule::delete_entity(Origin::signed(test_ed25519())));
	});
}

#[test]
fn test_delete_entity_fails_if_signed_by_someone_else() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";
		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature.clone()));

		assert_noop!(
			TfgridModule::delete_entity(Origin::signed(bob())),
			Error::<TestRuntime>::EntityNotExists
		);
	});
}

#[test]
fn test_create_twin_works() {
	ExternalityBuilder::build().execute_with(|| {
		let ip = "10.2.3.3";
		assert_ok!(TfgridModule::create_twin(Origin::signed(test_ed25519()), ip.as_bytes().to_vec()));
	});
}

#[test]
fn test_delete_twin_works() {
	ExternalityBuilder::build().execute_with(|| {
		let ip = "10.2.3.3";
		assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));

		let twin_id = 1;
		assert_ok!(TfgridModule::delete_twin(Origin::signed(alice()), twin_id));
	});
}

#[test]
fn test_add_entity_to_twin() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		// Someone first creates an entity
		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature.clone()));

		// Bob creates an anonymous twin
		let ip = "10.2.3.3";
		assert_ok!(TfgridModule::create_twin(Origin::signed(bob()), ip.as_bytes().to_vec()));

		// Signature of the entityid (1) and twinid (1) signed with test_ed25519 account
		let signature = sign_add_entity_to_twin(1, 1);
		
		let twin_id = 1;
		let entity_id = 1;
		
		// Bob adds someone as entity to his twin
		assert_ok!(TfgridModule::add_twin_entity(Origin::signed(bob()), twin_id, entity_id, signature));
	});
}

#[test]
fn test_add_entity_to_twin_fails_with_invalid_signature() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		// Someone first creates an entity
		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature.clone()));

		let ip = "10.2.3.3";
		assert_ok!(TfgridModule::create_twin(Origin::signed(bob()), ip.as_bytes().to_vec()));

		// Add Alice as entity to bob's twin

		// Signature of the entityid (1) and twinid (2) signed with test_ed25519 account
		let signature = sign_add_entity_to_twin(1, 2);
		
		let twin_id = 1;
		let entity_id = 1;
		
		assert_noop!(
			TfgridModule::add_twin_entity(Origin::signed(bob()), twin_id, entity_id, signature),
			Error::<TestRuntime>::EntitySignatureDoesNotMatch
		);
	});
}

#[test]
fn test_add_entity_to_twin_fails_if_entity_is_added_twice() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		// Someone first creates an entity
		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature.clone()));

		let ip = "10.2.3.3";
		assert_ok!(TfgridModule::create_twin(Origin::signed(bob()), ip.as_bytes().to_vec()));

		// Add Alice as entity to bob's twin

		// Signature of the entityid (1) and twinid (1) signed with test_ed25519 account
		let signature = sign_add_entity_to_twin(1, 1);
		
		let twin_id = 1;
		let entity_id = 1;
		
		assert_ok!(TfgridModule::add_twin_entity(Origin::signed(bob()), twin_id, entity_id, signature.clone()));
		
		assert_noop!(
			TfgridModule::add_twin_entity(Origin::signed(bob()), twin_id, entity_id, signature),
			Error::<TestRuntime>::EntityWithSignatureAlreadyExists
		);
	});
}

#[test]
fn test_create_twin_double_fails() {
	ExternalityBuilder::build().execute_with(|| {
		let ip = "10.2.3.3";
		assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));

		assert_noop!(
			TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()),
			Error::<TestRuntime>::TwinWithPubkeyExists
		);
	});
}

#[test]
fn test_create_farm_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		// Someone first creates an entity
		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature.clone()));

		let ip = "10.2.3.3";
		assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));

		let farm_name = "test_farm";
		let mut pub_ips = Vec::new();
		pub_ips.push(super::types::PublicIP{
			ip: "1.1.1.0".as_bytes().to_vec(),
			gateway: "1.1.1.1".as_bytes().to_vec(),
			contract_id: 0
		});

		assert_ok!(TfgridModule::create_farm(Origin::signed(alice()), farm_name.as_bytes().to_vec(), super::types::CertificationType::Diy, 0, 0, pub_ips));
	});
}

#[test]
fn test_create_farm_with_double_ip_fails() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		// Someone first creates an entity
		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature.clone()));

		let ip = "10.2.3.3";
		assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));

		let farm_name = "test_farm";
		let mut pub_ips = Vec::new();
		pub_ips.push(super::types::PublicIP{
			ip: "1.1.1.0".as_bytes().to_vec(),
			gateway: "1.1.1.1".as_bytes().to_vec(),
			contract_id: 0
		});
		pub_ips.push(super::types::PublicIP{
			ip: "1.1.1.0".as_bytes().to_vec(),
			gateway: "1.1.1.1".as_bytes().to_vec(),
			contract_id: 0
		});

		assert_noop!(
			TfgridModule::create_farm(Origin::signed(alice()), farm_name.as_bytes().to_vec(), super::types::CertificationType::Diy, 0, 0, pub_ips),
			Error::<TestRuntime>::IpExists
		);
	});
}

#[test]
fn test_adding_ip_to_farm_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		// Someone first creates an entity
		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature.clone()));

		let ip = "10.2.3.3";
		assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));

		let farm_name = "test_farm";
		let mut pub_ips = Vec::new();
		pub_ips.push(super::types::PublicIP{
			ip: "1.1.1.0".as_bytes().to_vec(),
			gateway: "1.1.1.1".as_bytes().to_vec(),
			contract_id: 0
		});

		assert_ok!(TfgridModule::create_farm(Origin::signed(alice()), farm_name.as_bytes().to_vec(), super::types::CertificationType::Diy, 0, 0, pub_ips));

		assert_ok!(TfgridModule::add_farm_ip(Origin::signed(alice()), 1, "1.1.1.2".as_bytes().to_vec(), "1.1.1.1".as_bytes().to_vec()));
	});
}

#[test]
fn test_adding_ip_duplicate_to_farm_fails() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		// Someone first creates an entity
		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature.clone()));

		let ip = "10.2.3.3";
		assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));

		let farm_name = "test_farm";
		let mut pub_ips = Vec::new();
		pub_ips.push(super::types::PublicIP{
			ip: "1.1.1.0".as_bytes().to_vec(),
			gateway: "1.1.1.1".as_bytes().to_vec(),
			contract_id: 0
		});

		assert_ok!(TfgridModule::create_farm(Origin::signed(alice()), farm_name.as_bytes().to_vec(), super::types::CertificationType::Diy, 0, 0, pub_ips));

		assert_ok!(TfgridModule::add_farm_ip(Origin::signed(alice()), 1, "1.1.1.2".as_bytes().to_vec(), "1.1.1.1".as_bytes().to_vec()));

		assert_noop!(
			TfgridModule::add_farm_ip(Origin::signed(alice()), 1, "1.1.1.2".as_bytes().to_vec(), "1.1.1.1".as_bytes().to_vec()),
			Error::<TestRuntime>::IpExists
		);
	});
}


#[test]
fn test_update_twin_works() {
	ExternalityBuilder::build().execute_with(|| {
		let mut ip = "some_ip";
		assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));

		ip = "some_other_ip";
		assert_ok!(TfgridModule::update_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));
	});
}

#[test]
fn test_update_twin_fails_if_signed_by_someone_else() {
	ExternalityBuilder::build().execute_with(|| {
		let mut ip = "some_ip";
		assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));

		ip = "some_other_ip";
		assert_noop!(
			TfgridModule::update_twin(Origin::signed(bob()), ip.as_bytes().to_vec()),
			Error::<TestRuntime>::TwinNotExists
		);
	});
}


#[test]
fn test_create_farm_with_same_name_fails() {
	ExternalityBuilder::build().execute_with(|| {		
		let name = "foobar";

		// Someone first creates an entity
		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature.clone()));

		let ip = "10.2.3.3";
		assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));

		let farm_name = "test_farm";
		let mut pub_ips = Vec::new();
		pub_ips.push(super::types::PublicIP{
			ip: "1.1.1.0".as_bytes().to_vec(),
			gateway: "1.1.1.1".as_bytes().to_vec(),
			contract_id: 0
		});
		assert_ok!(TfgridModule::create_farm(Origin::signed(alice()), farm_name.as_bytes().to_vec(), super::types::CertificationType::Diy, 0, 0, pub_ips.clone()));

		assert_noop!(
			TfgridModule::create_farm(Origin::signed(alice()), farm_name.as_bytes().to_vec(), super::types::CertificationType::Diy, 0, 0, pub_ips),
			Error::<TestRuntime>::FarmExists
		);
	});
}

#[test]
fn create_node_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		// Someone first creates an entity
		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature.clone()));

		let ip = "10.2.3.3";
		assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));

		let farm_name = "test_farm";
		let mut pub_ips = Vec::new();
		pub_ips.push(super::types::PublicIP{
			ip: "1.1.1.0".as_bytes().to_vec(),
			gateway: "1.1.1.1".as_bytes().to_vec(),
			contract_id: 0
		});
		assert_ok!(TfgridModule::create_farm(Origin::signed(alice()), farm_name.as_bytes().to_vec(), super::types::CertificationType::Diy, 0, 0, pub_ips.clone()));


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

		assert_ok!(TfgridModule::create_node(Origin::signed(alice()), 1, resources, location, 0, 0, None));
	});
}

#[test]
fn node_report_uptime_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		// Someone first creates an entity
		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature.clone()));

		let ip = "10.2.3.3";
		assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));

		let farm_name = "test_farm";
		let mut pub_ips = Vec::new();
		pub_ips.push(super::types::PublicIP{
			ip: "1.1.1.0".as_bytes().to_vec(),
			gateway: "1.1.1.1".as_bytes().to_vec(),
			contract_id: 0
		});
		assert_ok!(TfgridModule::create_farm(Origin::signed(alice()), farm_name.as_bytes().to_vec(), super::types::CertificationType::Diy, 0, 0, pub_ips.clone()));

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

		assert_ok!(TfgridModule::create_node(Origin::signed(alice()), 1, resources, location, 0, 0, None));

		Timestamp::set_timestamp(1628082000);
		assert_ok!(TfgridModule::report_uptime(Origin::signed(alice()), 500));

		let node = TfgridModule::nodes(1);
		assert_eq!(node.uptime, 500);
	});
}

#[test]
fn create_node_with_same_pubkey_fails() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		// Someone first creates an entity
		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature.clone()));

		let ip = "10.2.3.3";
		assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));

		let farm_name = "test_farm";
		let mut pub_ips = Vec::new();
		pub_ips.push(super::types::PublicIP{
			ip: "1.1.1.0".as_bytes().to_vec(),
			gateway: "1.1.1.1".as_bytes().to_vec(),
			contract_id: 0
		});
		assert_ok!(TfgridModule::create_farm(Origin::signed(alice()), farm_name.as_bytes().to_vec(), super::types::CertificationType::Diy, 0, 0, pub_ips.clone()));


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

		assert_ok!(TfgridModule::create_node(Origin::signed(alice()), 1, resources, location.clone(), 0, 0, None));

		assert_noop!(
			TfgridModule::create_node(Origin::signed(alice()), 1, resources, location, 0, 0, None),
			Error::<TestRuntime>::NodeWithTwinIdExists
		);
	});
}

#[test]
fn create_farming_policy_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "test".as_bytes().to_vec();

		assert_ok!(TfgridModule::create_farming_policy(RawOrigin::Root.into(), name, 12, 15, 10, 8, super::types::CertificationType::Diy));
	});
}

#[test]
fn node_auto_attach_farming_policy() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		// Someone first creates an entity
		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature.clone()));

		let ip = "10.2.3.3";
		assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));

		let farm_name = "test_farm";
		let mut pub_ips = Vec::new();
		pub_ips.push(super::types::PublicIP{
			ip: "1.1.1.0".as_bytes().to_vec(),
			gateway: "1.1.1.1".as_bytes().to_vec(),
			contract_id: 0
		});
		assert_ok!(TfgridModule::create_farm(Origin::signed(alice()), farm_name.as_bytes().to_vec(), super::types::CertificationType::Diy, 0, 0, pub_ips.clone()));

		// Create farming policies first
		let name = "d1_test".as_bytes().to_vec();
		assert_ok!(TfgridModule::create_farming_policy(RawOrigin::Root.into(), name, 12, 15, 10, 8, super::types::CertificationType::Diy));
		let name = "c1_test".as_bytes().to_vec();
		assert_ok!(TfgridModule::create_farming_policy(RawOrigin::Root.into(), name, 12, 15, 10, 8, super::types::CertificationType::Certified));
		let name = "d2_test".as_bytes().to_vec();
		assert_ok!(TfgridModule::create_farming_policy(RawOrigin::Root.into(), name, 12, 15, 10, 8, super::types::CertificationType::Diy));
		let name = "c2_test".as_bytes().to_vec();
		assert_ok!(TfgridModule::create_farming_policy(RawOrigin::Root.into(), name, 12, 15, 10, 8, super::types::CertificationType::Certified));

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

		assert_ok!(TfgridModule::create_node(Origin::signed(alice()), 1, resources, location, 0, 0, None));

		let node = TfgridModule::nodes(1);
		// farming policy set on the node should be 3
		// as we created the last DIY policy with id 3
		assert_eq!(node.farming_policy_id, 3);
	});
}