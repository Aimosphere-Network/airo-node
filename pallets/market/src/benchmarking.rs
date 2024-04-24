#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v2::*;
use frame_support::assert_ok;
use frame_system::RawOrigin;
use sp_std::prelude::*;

pub use airo_primitives::benchmarking::ModelFactory;

#[allow(unused)]
use crate::Pallet as AiroMarket;

use super::*;

const SEED: u32 = 0;

fn get_account<T: Config>(index: u32) -> T::AccountId {
    account("account", index, SEED)
}

fn create_order<T: Config>(consumer: T::AccountId) -> T::OrderId {
    assert_ok!(AiroMarket::<T>::order_create(
        RawOrigin::Signed(consumer).into(),
        T::BenchmarkHelper::get_model_id(),
        10_000,
    ));
    CurrentOrderId::<T>::get()
}

fn create_bid<T: Config>(provider: T::AccountId, order_id: T::OrderId) {
    let price_per_request = BalanceOf::<T>::from(10u32);
    assert_ok!(AiroMarket::<T>::bid_create(
        RawOrigin::Signed(provider).into(),
        order_id,
        price_per_request
    ));
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn order_create() {
        let caller: T::AccountId = whitelisted_caller();
        let model_id = T::BenchmarkHelper::get_model_id();
        let requests_total = 100;

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), model_id, requests_total);
    }

    #[benchmark]
    fn bid_create() {
        let consumer = get_account::<T>(1);
        let order_id = create_order::<T>(consumer);

        let caller: T::AccountId = whitelisted_caller();
        let price_per_request = BalanceOf::<T>::from(100u32);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), order_id, price_per_request);
    }

    #[benchmark]
    fn bid_accept() {
        let consumer: T::AccountId = whitelisted_caller();
        let order_id = create_order::<T>(consumer.clone());

        let provider = get_account::<T>(2);
        create_bid::<T>(provider.clone(), order_id);

        #[extrinsic_call]
        _(RawOrigin::Signed(consumer), order_id, provider);
    }

    impl_benchmark_test_suite!(AiroMarket, mock::new_test_ext(), mock::Test);
}
