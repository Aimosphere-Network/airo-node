#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::v2::*;
use frame_support::assert_ok;
use frame_system::RawOrigin;
use sp_std::prelude::*;

pub use airo_primitives::benchmarking::{ContentFactory, ModelFactory};

#[allow(unused)]
use crate::Pallet as AiroExecution;

use super::*;

const SEED: u32 = 0;

fn get_account<T: Config>(index: u32) -> T::AccountId {
    account("account", index, SEED)
}

fn prefund_account<T: Config>(account: &T::AccountId) {
    T::Currency::set_balance(account, BalanceOf::<T>::from(100_000_000u32));
}

fn create_agreement<T: Config>(
    consumer: T::AccountId,
    provider: T::AccountId,
    agreement_id: T::AgreementId,
) {
    assert_ok!(AiroExecution::<T>::create_agreement(
        consumer,
        provider,
        agreement_id,
        T::BenchmarkHelper::get_model_id(),
        BalanceOf::<T>::from(1_000u32),
        10,
    ));
}

fn create_request<T: Config>(
    consumer: T::AccountId,
    agreement_id: T::AgreementId,
) -> RequestsUsize {
    assert_ok!(AiroExecution::<T>::request_create(
        RawOrigin::Signed(consumer).into(),
        agreement_id,
        T::BenchmarkHelper::get_content_id(),
    ));

    Agreements::<T>::get(&agreement_id).unwrap().requests_count
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn request_create() {
        let caller: T::AccountId = whitelisted_caller();
        prefund_account::<T>(&caller);
        let provider = get_account::<T>(1);
        let agreement_id = T::AgreementId::default();
        create_agreement::<T>(caller.clone(), provider, agreement_id);

        #[extrinsic_call]
        _(RawOrigin::Signed(caller), agreement_id, T::BenchmarkHelper::get_content_id());
    }

    #[benchmark]
    fn response_create() {
        let consumer: T::AccountId = get_account::<T>(1);
        prefund_account::<T>(&consumer);
        let caller: T::AccountId = whitelisted_caller();
        let agreement_id = T::AgreementId::default();
        create_agreement::<T>(consumer.clone(), caller.clone(), agreement_id);
        let request_index = create_request::<T>(consumer, agreement_id);

        #[extrinsic_call]
        _(
            RawOrigin::Signed(caller),
            agreement_id,
            request_index,
            T::BenchmarkHelper::get_content_id(),
        );
    }

    impl_benchmark_test_suite!(AiroExecution, mock::new_test_ext(), mock::Test);
}
