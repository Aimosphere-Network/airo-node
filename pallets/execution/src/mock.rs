use frame_support::{
    derive_impl,
    traits::{ConstU16, ConstU32, ConstU64},
    BoundedVec,
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

use crate as pallet_execution;

type Block = frame_system::mocking::MockBlock<Test>;
pub type AccountId = u64;
pub type Balance = u64;
pub type ModelId = BoundedVec<u8, ConstU32<128>>;
pub type AgreementId = u32;
pub type ContentId = H256;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        AiroExecution: pallet_execution,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = ();
    type WeightInfo = ();
    type Balance = Balance;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU64<1>;
    type AccountStore = System;
    type ReserveIdentifier = [u8; 8];
    type FreezeIdentifier = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type MaxFreezes = ();
}

#[cfg(feature = "runtime-benchmarks")]
pub struct AiroExecutionBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl crate::benchmarking::ModelFactory<ModelId> for AiroExecutionBenchmarkHelper {
    fn get_model_id() -> ModelId {
        sp_core::bounded_vec![1; 128]
    }
}

#[cfg(feature = "runtime-benchmarks")]
impl crate::benchmarking::ContentFactory<ContentId> for AiroExecutionBenchmarkHelper {
    fn get_content_id() -> ContentId {
        ContentId::random()
    }
}

impl pallet_execution::Config for Test {
    type WeightInfo = ();
    type RuntimeEvent = RuntimeEvent;
    type RuntimeHoldReason = RuntimeHoldReason;
    type Currency = Balances;
    type AgreementId = AgreementId;
    type ModelId = ModelId;
    type ContentId = ContentId;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = AiroExecutionBenchmarkHelper;
}

pub const INITIAL_BALANCE: Balance = 1_000_000_000;
pub const CONSUMER_NO_BALANCE: AccountId = 0;
pub const CONSUMER_1: AccountId = 1;
pub const CONSUMER_2: AccountId = 2;
pub const PROVIDER_1: AccountId = 11;
pub const PROVIDER_2: AccountId = 12;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(CONSUMER_1, INITIAL_BALANCE), (CONSUMER_2, INITIAL_BALANCE)],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(storage);
    // Go past genesis block so events get deposited
    ext.execute_with(|| System::set_block_number(1));
    ext
}
