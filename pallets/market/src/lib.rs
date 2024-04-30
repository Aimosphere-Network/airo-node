//! # Market Pallet

// We make sure this pallet uses `no_std` for compiling to Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    pallet_prelude::*,
    traits::fungible::{hold::Mutate as FunHoldMutate, Inspect as FunInspect, Mutate as FunMutate},
};
use frame_system::pallet_prelude::*;
use sp_runtime::traits::{One, Zero};

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

        /// Model ID type.
        type ModelId: Member + Parameter + MaxEncodedLen + Default;

        /// Order ID type.
        type OrderId: Member + Parameter + MaxEncodedLen + One + Zero + Default + Copy;

        /// Used to operate on agreements.
        type AgreementManagement: AgreementManagement<
            AccountId = Self::AccountId,
            OrderId = Self::OrderId,
            ModelId = Self::ModelId,
            Balance = BalanceOf<Self>,
        >;

        #[cfg(feature = "runtime-benchmarks")]
        type BenchmarkHelper: benchmarking::ModelFactory<Self::ModelId>;
    }

    /// The current order ID. This is incremented when a new order is created.
    #[pallet::storage]
    pub type CurrentOrderId<T: Config> = StorageValue<_, T::OrderId, ValueQuery>;

    /// Orders currently existing in the network.
    #[pallet::storage]
    pub type Orders<T: Config> = StorageMap<_, Blake2_128Concat, T::OrderId, OrderDetails<T>>;

    // TODO. Remove this storage in favor of Backend.
    /// Orders created by a consumer.
    #[pallet::storage]
    pub type ConsumerOrders<T: Config> =
        StorageDoubleMap<_, Twox64Concat, Consumer<T>, Blake2_128Concat, T::OrderId, ()>;

    // TODO. Remove this storage in favor of Backend.
    /// Orders bid by a provider.
    #[pallet::storage]
    pub type ProviderOrders<T: Config> =
        StorageDoubleMap<_, Twox64Concat, Provider<T>, Blake2_128Concat, T::OrderId, ()>;

    /// Bids currently existing in the network.
    #[pallet::storage]
    pub type OrderBids<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::OrderId, Twox64Concat, Provider<T>, BidDetails<T>>;

    /// A reason for the Market pallet placing a hold on funds.
    #[pallet::composite_enum]
    pub enum HoldReason {
        // TODO. Implement service deposits
        /// Consumer's service deposit.
        ConsumerServiceDeposit,
    }

    /// Events.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new order has been created.
        OrderCreated {
            /// The order ID.
            order_id: T::OrderId,
            /// The model ID.
            model_id: T::ModelId,
        },
        /// A bid has been created.
        BidCreated {
            /// The order ID.
            order_id: T::OrderId,
            /// The provider.
            provider: T::AccountId,
            /// The price per request.
            price_per_request: BalanceOf<T>,
        },
        /// A bid has been accepted.
        BidAccepted {
            /// The order ID.
            order_id: T::OrderId,
            /// The provider.
            provider: T::AccountId,
        },
    }

    /// Errors.
    #[pallet::error]
    pub enum Error<T> {
        /// Order is not found.
        OrderNotFound,
        /// Order is invalid.
        OrderInvalid,
        /// Bid is not found.
        BidNotFound,
        /// Bid already exists.
        BidAlreadyExists,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Creates a new order on the market.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::order_create())]
        pub fn order_create(
            origin: OriginFor<T>,
            model_id: T::ModelId,
            #[pallet::compact]
            requests_total: RequestsUsize,
        ) -> DispatchResult {
            let consumer = ensure_signed(origin)?;

            let order_details = OrderDetails::new(consumer, model_id.clone(), requests_total);
            ensure!(order_details.is_valid(), Error::<T>::OrderInvalid);

            let order_id = Order::<T>::insert(order_details);

            Self::deposit_event(Event::OrderCreated { order_id, model_id });
            Ok(())
        }

        /// Executed by a provider to create a bid on an order.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::bid_create())]
        pub fn bid_create(
            origin: OriginFor<T>,
            order_id: T::OrderId,
            price_per_request: BalanceOf<T>,
        ) -> DispatchResult {
            let provider = ensure_signed(origin)?;

            ensure!(Order::<T>::exists(order_id), Error::<T>::OrderNotFound);
            ensure!(!Bid::<T>::exists(order_id, &provider), Error::<T>::BidAlreadyExists);

            let bid_details = BidDetails::new(provider.clone(), price_per_request);
            Bid::<T>::insert(order_id, &provider, bid_details);

            Self::deposit_event(Event::BidCreated { order_id, provider, price_per_request });
            Ok(())
        }

        /// Executed by a consumer to accept a bid on an order.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::bid_accept())]
        pub fn bid_accept(
            origin: OriginFor<T>,
            order_id: T::OrderId,
            provider: T::AccountId,
        ) -> DispatchResult {
            let consumer = ensure_signed(origin)?;

            let order = Orders::<T>::get(order_id).ok_or(Error::<T>::OrderNotFound)?;
            ensure!(order.is_owned_by(&consumer), Error::<T>::OrderInvalid);
            let bid = OrderBids::<T>::get(order_id, &provider).ok_or(Error::<T>::BidNotFound)?;

            T::AgreementManagement::create_agreement(
                consumer,
                provider.clone(),
                order_id,
                order.model_id,
                bid.price_per_request,
                order.requests_total,
            )?;
            Order::<T>::remove(order_id);

            Self::deposit_event(Event::BidAccepted { order_id, provider });
            Ok(())
        }
    }
}
