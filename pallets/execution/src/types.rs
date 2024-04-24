use airo_primitives::RequestsUsize;
use frame_support::traits::tokens::{Fortitude::Polite, Precision::BestEffort, Restriction::Free};

use crate::*;

/// Type alias for the balance type from the runtime.
pub type BalanceOf<T> =
    <<T as Config>::Currency as FunInspect<<T as frame_system::Config>::AccountId>>::Balance;

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type Consumer<T> = AccountIdOf<T>;
pub type Provider<T> = AccountIdOf<T>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct AgreementDetails<T: Config> {
    pub consumer: T::AccountId,
    pub provider: T::AccountId,
    pub model_id: T::ModelId,
    pub price_per_request: BalanceOf<T>,
    pub requests_count: RequestsUsize,
    pub requests_total: RequestsUsize,
}

impl<T: Config> AgreementDetails<T> {
    pub fn new(
        consumer: T::AccountId,
        provider: T::AccountId,
        model_id: T::ModelId,
        price_per_request: BalanceOf<T>,
        requests_total: RequestsUsize,
    ) -> Self {
        Self { consumer, provider, model_id, price_per_request, requests_count: 0, requests_total }
    }

    pub fn is_consumer(&self, consumer: &Consumer<T>) -> bool {
        self.consumer == *consumer
    }

    pub fn is_provider(&self, provider: &Provider<T>) -> bool {
        self.provider == *provider
    }

    pub fn next_request_index(&mut self) -> Result<RequestsUsize, Error<T>> {
        if self.requests_count == self.requests_total {
            Err(Error::<T>::RequestNotAllowed)
        } else {
            self.requests_count += 1;
            Ok(self.requests_count)
        }
    }

    pub fn request_exists(&self, request_index: RequestsUsize) -> bool {
        request_index <= self.requests_count
    }
}

// Payments
impl<T: Config> AgreementDetails<T> {
    pub fn hold_consumer_prepayment(&self) -> DispatchResult {
        T::Currency::hold(
            &HoldReason::ConsumerPrepayment.into(),
            &self.consumer,
            self.price_per_request.saturating_mul(self.requests_total.into()),
        )
    }

    pub fn transfer_payment(&self) -> Result<BalanceOf<T>, DispatchError> {
        T::Currency::transfer_on_hold(
            &HoldReason::ConsumerPrepayment.into(),
            &self.consumer,
            &self.provider,
            self.price_per_request,
            BestEffort,
            Free,
            Polite,
        )
    }
}
