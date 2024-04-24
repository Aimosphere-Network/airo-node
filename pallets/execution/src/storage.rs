use airo_primitives::RequestsUsize;

use crate::*;

pub struct Agreement<T>(PhantomData<T>);

impl<T: Config> Agreement<T> {
    pub fn insert(agreement_id: T::AgreementId, agreement: AgreementDetails<T>) {
        ConsumerAgreements::<T>::insert(&agreement.consumer, agreement_id, ());
        ProviderAgreements::<T>::insert(&agreement.provider, agreement_id, ());
        Agreements::<T>::insert(agreement_id, agreement);
    }
}

pub struct Response<T>(PhantomData<T>);

impl<T: Config> Response<T> {
    pub fn exists(agreement_id: T::AgreementId, request_index: RequestsUsize) -> bool {
        Responses::<T>::contains_key(agreement_id, request_index)
    }
}
