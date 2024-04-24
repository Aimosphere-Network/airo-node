use frame_support::dispatch::DispatchResult;

use crate::RequestsUsize;

pub trait AgreementManagement {
    type AccountId;
    type OrderId;
    type ModelId;
    type Balance;

    fn create_agreement(
        consumer: Self::AccountId,
        provider: Self::AccountId,
        order_id: Self::OrderId,
        model_id: Self::ModelId,
        price_per_request: Self::Balance,
        requests_total: RequestsUsize,
    ) -> DispatchResult;
}
