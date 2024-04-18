use core::ops::Add;

use frame_support::StorageValue;
use scale_codec::FullCodec;

use crate::*;

// TODO. Move to common.
pub trait IncrementalStorageValue
where
    Self: StorageValue<Self::Id, Query = Self::Id>,
{
    type Id: FullCodec + Copy + One + Zero;

    fn next() -> Self::Id {
        Self::mutate(|value| {
            *value = value.add(One::one());
            *value
        })
    }
}

impl<T: Config> IncrementalStorageValue for CurrentOrderId<T> {
    type Id = T::OrderId;
}

impl<T: Config> Pallet<T> {
    pub fn insert_order(order: OrderDetails<T>) -> T::OrderId {
        let order_id = CurrentOrderId::<T>::next();
        ConsumerOrders::<T>::insert(&order.consumer, order_id, ());
        Orders::<T>::insert(order_id, order);
        order_id
    }

    pub fn insert_bid(order_id: T::OrderId, provider: &Provider<T>, bid: Bid<T>) {
        ProviderOrders::<T>::insert(provider, order_id, ());
        OrderBids::<T>::insert(order_id, provider, bid);
    }

    pub fn remove_order(order_id: T::OrderId) {
        if let Some(OrderDetails { consumer, .. }) = Orders::<T>::take(order_id) {
            ConsumerOrders::<T>::remove(&consumer, order_id);

            OrderBids::<T>::drain_prefix(order_id).for_each(|(provider, _)| {
                ProviderOrders::<T>::remove(provider, order_id);
            });
        }
    }
}
