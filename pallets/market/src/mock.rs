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

use crate as pallet_market;

type Block = frame_system::mocking::MockBlock<Test>;

type ModelId = BoundedVec<u8, ConstU32<128>>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        Balances: pallet_balances,
        AIMarket: pallet_market,
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
    type AccountId = u64;
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
    type Balance = u64;
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
pub struct AIMarketBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl pallet_market::benchmarking::BenchmarkHelper<ModelId> for AIMarketBenchmarkHelper {
    fn get_model_id() -> ModelId {
        sp_core::bounded_vec![1; 128]
    }
}

impl pallet_market::Config for Test {
    type WeightInfo = ();
    type RuntimeEvent = RuntimeEvent;
    type RuntimeHoldReason = RuntimeHoldReason;
    type Currency = Balances;
    type ModelId = ModelId;
    type OrderId = u32;

    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = AIMarketBenchmarkHelper;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}
