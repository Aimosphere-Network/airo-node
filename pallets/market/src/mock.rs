use frame_support::{
    derive_impl,
    dispatch::DispatchResult,
    traits::{ConstU16, ConstU32, ConstU64},
    BoundedVec,
};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};

use airo_primitives::agreement::AgreementManagement;
use airo_primitives::RequestsUsize;

use crate as pallet_market;

type Block = frame_system::mocking::MockBlock<Test>;
pub type AccountId = u64;
pub type Balance = u64;
type ModelId = BoundedVec<u8, ConstU32<128>>;
pub type OrderId = u32;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        AiroMarket: pallet_market,
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
    type DoneSlashHandler = ();
}

#[cfg(feature = "runtime-benchmarks")]
pub struct AiroMarketBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl crate::benchmarking::ModelFactory<ModelId> for AiroMarketBenchmarkHelper {
    fn get_model_id() -> ModelId {
        sp_core::bounded_vec![1; 128]
    }
}

pub struct MockAgreementManagement;

impl AgreementManagement for MockAgreementManagement {
    type AccountId = AccountId;
    type OrderId = OrderId;
    type ModelId = ModelId;
    type Balance = Balance;

    fn create_agreement(
        _consumer: Self::AccountId,
        _provider: Self::AccountId,
        _order_id: Self::OrderId,
        _model_id: Self::ModelId,
        _price_per_request: Self::Balance,
        _requests_total: RequestsUsize,
    ) -> DispatchResult {
        Ok(())
    }
}

impl pallet_market::Config for Test {
    type WeightInfo = ();
    type RuntimeEvent = RuntimeEvent;
    type RuntimeHoldReason = RuntimeHoldReason;
    type Currency = Balances;
    type ModelId = ModelId;
    type OrderId = OrderId;
    type AgreementManagement = MockAgreementManagement;
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = AiroMarketBenchmarkHelper;
}

pub const CONSUMER_1: AccountId = 1;
pub const CONSUMER_2: AccountId = 2;
pub const PROVIDER_1: AccountId = 11;
pub const PROVIDER_2: AccountId = 12;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
    let mut ext = sp_io::TestExternalities::new(storage);
    // Go past genesis block so events get deposited
    ext.execute_with(|| System::set_block_number(1));
    ext
}
