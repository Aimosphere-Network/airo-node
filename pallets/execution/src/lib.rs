//! # Execution Pallet

// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    pallet_prelude::*,
    traits::fungible::{hold::Mutate as FunHoldMutate, Inspect as FunInspect, Mutate as FunMutate},
};
use frame_system::pallet_prelude::*;
use sp_runtime::Saturating;

use airo_primitives::{agreement::AgreementManagement, RequestsUsize};
pub use pallet::*;
use storage::*;
use types::*;
pub use weights::*;

// FRAME pallets require their own "mock runtimes" to be able to run unit tests. This module
// contains a mock runtime specific for testing this pallet's functionality.
#[cfg(test)]
mod mock;

// This module contains the unit tests for this pallet.
// Learn about pallet unit testing here: https://docs.substrate.io/test/unit-testing/
#[cfg(test)]
mod tests;

// Every callable function or "dispatchable" a pallet exposes must have weight values that correctly
// estimate a dispatchable's execution time. The benchmarking module is used to calculate weights
// for each dispatchable and generates this pallet's weight.rs file. Learn more about benchmarking here: https://docs.substrate.io/test/benchmark/
#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

mod storage;
mod types;
pub mod weights;

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: WeightInfo;

        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Overarching hold reason.
        type RuntimeHoldReason: From<HoldReason>;

        /// Currency type that this works on.
        type Currency: FunMutate<Self::AccountId>
            + FunHoldMutate<Self::AccountId, Reason = Self::RuntimeHoldReason>;

        /// Agreement ID type.
        type AgreementId: Member + Parameter + MaxEncodedLen + Copy + Default;

        /// Model ID type.
        type ModelId: Member + Parameter + MaxEncodedLen;

        /// Content ID type.
        type ContentId: Member + Parameter + MaxEncodedLen;

        #[cfg(feature = "runtime-benchmarks")]
        type BenchmarkHelper: benchmarking::ModelFactory<Self::ModelId>
            + benchmarking::ContentFactory<Self::ContentId>;
    }

    /// Agreements currently existing in the network.
    #[pallet::storage]
    pub type Agreements<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AgreementId, AgreementDetails<T>>;

    // TODO. Remove this storage in favor of Backend.
    /// Agreements referencing a consumer.
    #[pallet::storage]
    pub type ConsumerAgreements<T: Config> =
        StorageDoubleMap<_, Twox64Concat, Consumer<T>, Blake2_128Concat, T::AgreementId, ()>;

    // TODO. Remove this storage in favor of Backend.
    /// Agreements referencing a provider.
    #[pallet::storage]
    pub type ProviderAgreements<T: Config> =
        StorageDoubleMap<_, Twox64Concat, Provider<T>, Blake2_128Concat, T::AgreementId, ()>;

    /// Requests currently existing in the network.
    /// The key is a pair of Agreement ID and Request Index.
    #[pallet::storage]
    pub type Requests<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AgreementId,
        Blake2_128Concat,
        RequestsUsize,
        T::ContentId,
    >;

    /// Responses to the requests.
    #[pallet::storage]
    pub type Responses<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AgreementId,
        Blake2_128Concat,
        RequestsUsize,
        T::ContentId,
    >;

    /// A reason for the Execution pallet placing a hold on funds.
    #[pallet::composite_enum]
    pub enum HoldReason {
        /// Consumer's prepayment.
        ConsumerPrepayment,
    }

    /// Events.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new agreement has been created.
        AgreementCreated {
            /// The agreement ID, which is also the order ID.
            agreement_id: T::AgreementId,
        },
        /// A request has been created.
        RequestCreated {
            /// The agreement ID.
            agreement_id: T::AgreementId,
            /// The request index.
            request_index: RequestsUsize,
            /// The content ID.
            content_id: T::ContentId,
        },
        /// A response has been created.
        ResponseCreated {
            /// The agreement ID.
            agreement_id: T::AgreementId,
            /// The request index.
            request_index: RequestsUsize,
            /// The content ID.
            content_id: T::ContentId,
        },
    }

    /// Errors.
    #[pallet::error]
    pub enum Error<T> {
        /// Agreement is not found.
        AgreementNotFound,
        /// Agreement is invalid.
        AgreementInvalid,
        /// Request is not allowed. E.g. all requests have been used.
        RequestNotAllowed,
        /// Request is not found.
        RequestNotFound,
        /// Response is already exists.
        ResponseAlreadyExists,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::request_create())]
        pub fn request_create(
            origin: OriginFor<T>,
            agreement_id: T::AgreementId,
            content_id: T::ContentId,
        ) -> DispatchResult {
            let consumer = ensure_signed(origin)?;

            let request_index =
                Agreements::<T>::try_mutate(agreement_id, |agreement| -> Result<_, Error<T>> {
                    let agreement = agreement.as_mut().ok_or(Error::<T>::AgreementNotFound)?;
                    ensure!(agreement.is_consumer(&consumer), Error::<T>::AgreementInvalid);

                    let request_index = agreement.next_request_index()?;
                    Requests::<T>::insert(agreement_id, request_index, content_id.clone());
                    Ok(request_index)
                })?;

            Self::deposit_event(Event::<T>::RequestCreated {
                agreement_id,
                request_index,
                content_id,
            });
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::response_create())]
        pub fn response_create(
            origin: OriginFor<T>,
            agreement_id: T::AgreementId,
            #[pallet::compact] request_index: RequestsUsize,
            content_id: T::ContentId,
        ) -> DispatchResult {
            let provider = ensure_signed(origin)?;

            let agreement =
                Agreements::<T>::get(agreement_id).ok_or(Error::<T>::AgreementNotFound)?;
            ensure!(agreement.is_provider(&provider), Error::<T>::AgreementInvalid);
            ensure!(agreement.request_exists(request_index), Error::<T>::RequestNotFound);
            ensure!(
                !Response::<T>::exists(agreement_id, request_index),
                Error::<T>::ResponseAlreadyExists
            );

            let _ = agreement.transfer_payment()?;
            Responses::<T>::insert(agreement_id, request_index, content_id.clone());

            Self::deposit_event(Event::<T>::ResponseCreated {
                agreement_id,
                request_index,
                content_id,
            });
            Ok(())
        }
    }
}

impl<T: Config> AgreementManagement for Pallet<T> {
    type AccountId = T::AccountId;
    // Use order ID as the agreement ID for new agreements.
    type OrderId = T::AgreementId;
    type ModelId = T::ModelId;
    type Balance = BalanceOf<T>;

    fn create_agreement(
        consumer: Self::AccountId,
        provider: Self::AccountId,
        order_id: Self::OrderId,
        model_id: Self::ModelId,
        price_per_request: Self::Balance,
        requests_total: RequestsUsize,
    ) -> DispatchResult {
        let agreement =
            AgreementDetails::new(consumer, provider, model_id, price_per_request, requests_total);
        agreement.hold_consumer_prepayment()?;
        Agreement::<T>::insert(order_id, agreement);

        Self::deposit_event(Event::<T>::AgreementCreated { agreement_id: order_id });
        Ok(())
    }
}
