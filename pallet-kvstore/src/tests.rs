use crate::{self as pallet_kvstore, Config, Error, RawEvent};
use frame_support::{assert_err, assert_ok, construct_runtime, parameter_types};
use sp_core::H256;
use sp_io::TestExternalities;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
type Block = frame_system::mocking::MockBlock<TestRuntime>;

construct_runtime!(
    pub enum TestRuntime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        TFKVStoreModule: pallet_kvstore::{Module, Call, Storage, Event<T>},
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

#[test]
fn set() {
    ExternalityBuilder::build().execute_with(|| {
        let key = "name";
        let value = "nametest";
        //make sure Entery does not exists
        assert_err!(
            TFKVStoreModule::get(Origin::signed(1), key.as_bytes().to_vec()),
            Error::<TestRuntime>::NoValueStored
        );

        assert_ok!(TFKVStoreModule::set(
            Origin::signed(1),
            key.as_bytes().to_vec(),
            value.as_bytes().to_vec()
        ));
        let expected_event = Event::pallet_kvstore(RawEvent::EntrySet(
            1,
            key.as_bytes().to_vec(),
            value.as_bytes().to_vec(),
        ));

        assert_eq!(System::events()[0].event, expected_event,);
    })
}

#[test]
fn get() {
    ExternalityBuilder::build().execute_with(|| {
        let key = "Job";
        let value = "Engineer";
        assert_ok!(TFKVStoreModule::set(
            Origin::signed(1),
            key.as_bytes().to_vec(),
            value.as_bytes().to_vec()
        ));
        let expected_event = Event::pallet_kvstore(RawEvent::EntrySet(
            1,
            key.as_bytes().to_vec(),
            value.as_bytes().to_vec(),
        ));

        assert_eq!(System::events()[0].event, expected_event,);

        // Test getting the value

        assert_ok!(TFKVStoreModule::get(
            Origin::signed(1),
            key.as_bytes().to_vec()
        ));
        let expected_event = Event::pallet_kvstore(RawEvent::EntryGot(
            1,
            key.as_bytes().to_vec(),
            value.as_bytes().to_vec(),
        ));

        assert_eq!(System::events()[1].event, expected_event,);
    })
}

#[test]
fn delete() {
    ExternalityBuilder::build().execute_with(|| {
        let key = "Address";
        let value = "Cairo";
        assert_ok!(TFKVStoreModule::set(
            Origin::signed(1),
            key.as_bytes().to_vec(),
            value.as_bytes().to_vec()
        ));
        let expected_event = Event::pallet_kvstore(RawEvent::EntrySet(
            1,
            key.as_bytes().to_vec(),
            value.as_bytes().to_vec(),
        ));

        assert_eq!(System::events()[0].event, expected_event,);

        assert_ok!(TFKVStoreModule::delete(
            Origin::signed(1),
            key.as_bytes().to_vec()
        ));
        let expected_event = Event::pallet_kvstore(RawEvent::EntryTaken(
            1,
            key.as_bytes().to_vec(),
            value.as_bytes().to_vec(),
        ));

        assert_eq!(System::events()[1].event, expected_event,);

        assert_err!(
            TFKVStoreModule::get(Origin::signed(1), key.as_bytes().to_vec()),
            Error::<TestRuntime>::NoValueStored
        );
    })
}
