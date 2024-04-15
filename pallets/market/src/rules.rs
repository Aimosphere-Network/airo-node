use crate::*;

pub struct That<T>(PhantomData<T>);

impl<T: Config> That<T> {
    pub fn owns_order(consumer: &Consumer<T>, order_id: T::OrderId) -> bool {
        ConsumerOrders::<T>::contains_key(consumer, order_id)
    }

    pub fn order_valid(order: &OrderDetails<T>) -> bool {
        order.requests_total > 0 && order.model_id != T::ModelId::default()
    }

    pub fn order_exists(order_id: T::OrderId) -> bool {
        Orders::<T>::contains_key(order_id)
    }

    pub fn bid_exists(order_id: T::OrderId, provider: &Provider<T>) -> bool {
        OrderBids::<T>::contains_key(order_id, provider)
    }
}
