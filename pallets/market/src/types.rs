use crate::*;

/// Type alias for the balance type from the runtime.
pub type BalanceOf<T> =
    <<T as Config>::Currency as FunInspect<<T as frame_system::Config>::AccountId>>::Balance;

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type Consumer<T> = AccountIdOf<T>;
pub type Provider<T> = AccountIdOf<T>;

#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct OrderDetails<T: Config> {
    pub consumer: Consumer<T>,
    pub model_id: T::ModelId,
    pub requests_total: u32,
}

impl<T: Config> OrderDetails<T> {
    pub fn new(consumer: Consumer<T>, model_id: T::ModelId, requests_total: u32) -> Self {
        Self { consumer, model_id, requests_total }
    }
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, Debug, MaxEncodedLen, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct Bid<T: Config> {
    pub provider: Provider<T>,
    pub price_per_request: BalanceOf<T>,
}

impl<T: Config> Bid<T> {
    pub fn new(provider: Provider<T>, price_per_request: BalanceOf<T>) -> Self {
        Self { provider, price_per_request }
    }
}
