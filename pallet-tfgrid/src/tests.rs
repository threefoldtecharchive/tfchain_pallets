use crate::{self as tfgridModule, Config, RawEvent};
use frame_support::{assert_noop, assert_ok, construct_runtime, parameter_types};
use sp_io::TestExternalities;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

use sp_core::{H256, Pair, Public, ed25519, sr25519};

use sp_std::prelude::*;

use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::{
	MultiSignature,
};

use hex;

pub type Signature = MultiSignature;

pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

construct_runtime!(
	pub enum TestRuntime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		TfgridModule: tfgridModule::{Module, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}

impl frame_system::Config for TestRuntime {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Index = u64;
	type Call = Call;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

impl Config for TestRuntime {
	type Event = Event;
}

struct ExternalityBuilder;

impl ExternalityBuilder {
	pub fn build() -> TestExternalities {
		let storage = frame_system::GenesisConfig::default()
			.build_storage::<TestRuntime>()
			.unwrap();
		let mut ext = TestExternalities::from(storage);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
type AccountPublic = <MultiSignature as Verify>::Signer;


// industry dismiss casual gym gap music pave gasp sick owner dumb cost

/// Helper function to generate a crypto pair from seed
fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

fn get_from_seed_string<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

fn get_account_id_from_seed_string<TPublic: Public>(seed: &str) -> AccountId where
AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed_string::<TPublic>(seed)).into_account()
}

fn alice() -> AccountId {
	get_account_id_from_seed::<sr25519::Public>("Alice")
}

// fn alice_ed25519() -> AccountId {
// 	get_account_id_from_seed::<ed25519::Public>("Alice")
// }

fn test_ed25519() -> AccountId {
	get_account_id_from_seed_string::<ed25519::Public>("industry dismiss casual gym gap music pave gasp sick owner dumb cost")
}

fn bob() -> AccountId {
	get_account_id_from_seed::<sr25519::Public>("Bob")
}

fn sign_create_entity(name: Vec<u8>, country_id: u32, city_id: u32) -> Vec<u8> {
	let seed = hex::decode("59336423ee7af732b2d4a76e440651e33e5ba51540e5633535b9030492c2a6f6").unwrap();
	let pair = ed25519::Pair::from_seed_slice(&seed).unwrap();

	let mut message = vec![];
	message.extend_from_slice(&name);
	message.extend_from_slice(&country_id.to_be_bytes());
	message.extend_from_slice(&city_id.to_be_bytes());

	let signature = pair.sign(&message);

	// hex encode signature
	hex::encode(signature.0.to_vec()).into()
}

fn sign_add_entity_to_twin(entity_id: u32, twin_id: u32) -> Vec<u8> {
	let seed = hex::decode("59336423ee7af732b2d4a76e440651e33e5ba51540e5633535b9030492c2a6f6").unwrap();
	let pair = ed25519::Pair::from_seed_slice(&seed).unwrap();

	let mut message = vec![];
	message.extend_from_slice(&entity_id.to_be_bytes());
	message.extend_from_slice(&twin_id.to_be_bytes());

	let signature = pair.sign(&message);

	// hex encode signature
	hex::encode(signature.0.to_vec()).into()
}

#[test]
fn test_create_entity_works() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		let a = Origin::signed(1);
		assert_ok!(TfgridModule::create_entity(Origin::signed(1), a, name.as_bytes().to_vec(), 0,0, signature));
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
			Err::<TestRuntime>::EntityNotExists
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
			TfgridModule::Error::EntityWithNameExists
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
fn test_create_twin_double_works() {
	ExternalityBuilder::build().execute_with(|| {
		let ip = "10.2.3.3";
		assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));

		// Second time creating twin succeeds
		assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));
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

		let twin_id = 1;

		let name = "test_farm";

		let farm = super::types::Farm{
			version: 1,
			id: 0,
			name: name.as_bytes().to_vec(),
			twin_id,
			country_id: 0,
			city_id: 0,
			certification_type: super::types::CertificationType::None,
			pricing_policy_id: 0,
		};

		assert_ok!(TfgridModule::create_farm(Origin::signed(alice()), farm));
	});
}

#[test]
fn test_update_twin_works() {
	ExternalityBuilder::build().execute_with(|| {
		let mut ip = "some_ip";
		assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));

		let twin_id = 1;
		ip = "some_other_ip";
		assert_ok!(TfgridModule::update_twin(Origin::signed(alice()), twin_id, ip.as_bytes().to_vec()));
	});
}

#[test]
fn test_update_twin_fails_if_signed_by_someone_else() {
	ExternalityBuilder::build().execute_with(|| {
		let mut ip = "some_ip";
		assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));

		let twin_id = 1;
		ip = "some_other_ip";
		assert_noop!(
			TfgridModule::update_twin(Origin::signed(bob()), twin_id, ip.as_bytes().to_vec()),
			Error::<TestRuntime>::TwinNotExists
		);
	});
}


// #[test]
// fn test_create_farm_with_invalid_entity_id_fails() {
// 	ExternalityBuilder::build().execute_with(|| {		
// 		let farm_name = "test_farm";
		
// 		let twin_id = 1;
// 		let entity_id = 654;

// 		let farm = super::types::Farm{
// 			id: 0,
// 			name: farm_name.as_bytes().to_vec(),
// 			twin_id,
// 			entity_id,
// 			country_id: 0,
// 			city_id: 0,
// 			certification_type: super::types::CertificationType::None,
// 			pricing_policy_id: 0,
// 		};

// 		// Create farm with invalid entity-id
// 		assert_noop!(
// 			TfgridModule::create_farm(Origin::signed(alice()), farm),
// 			Error::<TestRuntime>::EntityNotExists
// 		);
// 	});
// }

#[test]
fn test_create_farm_with_invalid_twin_id_fails() {
	ExternalityBuilder::build().execute_with(|| {		
		let farm_name = "test_farm";

		let name = "foobar";
		// Someone first creates an entity
		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature.clone()));
		
		let twin_id = 5342433;

		let farm = super::types::Farm{
			version: 1,
			id: 0,
			name: farm_name.as_bytes().to_vec(),
			twin_id,
			country_id: 0,
			city_id: 0,
			certification_type: super::types::CertificationType::None,
			pricing_policy_id: 0,
		};

		// Create farm with invalid twin-id
		assert_noop!(
			TfgridModule::create_farm(Origin::signed(alice()), farm),
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

		let twin_id = 1;

		let farm_name = "test_farm";

		let farm = super::types::Farm{
			version: 1,
			id: 0,
			name: farm_name.as_bytes().to_vec(),
			twin_id,
			country_id: 0,
			city_id: 0,
			certification_type: super::types::CertificationType::None,
			pricing_policy_id: 0,
		};

		assert_ok!(TfgridModule::create_farm(Origin::signed(alice()), farm.clone()));

		assert_noop!(
			TfgridModule::create_farm(Origin::signed(alice()), farm),
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

		let twin_id = 1;

		let farm_name = "test_farm";

		let farm = super::types::Farm{
			version: 1,
			id: 0,
			name: farm_name.as_bytes().to_vec(),
			twin_id,
			country_id: 0,
			city_id: 0,
			certification_type: super::types::CertificationType::None,
			pricing_policy_id: 0,
		};

		assert_ok!(TfgridModule::create_farm(Origin::signed(alice()), farm));

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

		let node = super::types::Node {
			version: 1,
			id: 0,
			farm_id: 1,
			resources,
			location,
			city_id: 0,
			country_id: 0,
			address: alice(),
			pub_key: "some_node_id".as_bytes().to_vec(),
			role: super::types::Role::Node,
		};

		assert_ok!(TfgridModule::create_node(Origin::signed(alice()), node));
	});
}

#[test]
fn create_node_with_same_pubke_fails() {
	ExternalityBuilder::build().execute_with(|| {
		let name = "foobar";

		// Someone first creates an entity
		let signature = sign_create_entity(name.as_bytes().to_vec(), 0, 0);

		assert_ok!(TfgridModule::create_entity(Origin::signed(alice()), test_ed25519(), name.as_bytes().to_vec(), 0,0, signature.clone()));

		let ip = "10.2.3.3";
		assert_ok!(TfgridModule::create_twin(Origin::signed(alice()), ip.as_bytes().to_vec()));

		let twin_id = 1;

		let farm_name = "test_farm";

		let farm = super::types::Farm{
			version: 1,
			id: 0,
			name: farm_name.as_bytes().to_vec(),
			twin_id,
			country_id: 0,
			city_id: 0,
			certification_type: super::types::CertificationType::None,
			pricing_policy_id: 0,
		};

		assert_ok!(TfgridModule::create_farm(Origin::signed(alice()), farm));

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

		let node = super::types::Node {
			version: 1,
			id: 0,
			farm_id: 1,
			resources,
			location,
			city_id: 0,
			country_id: 0,
			address: alice(),
			pub_key: "some_node_id".as_bytes().to_vec(),
			role: super::types::Role::Node,
		};

		assert_ok!(TfgridModule::create_node(Origin::signed(alice()), node.clone()));

		assert_noop!(
			TfgridModule::create_node(Origin::signed(alice()), node),
			Error::<TestRuntime>::NodeWithPubkeyExists
		);
	});
}