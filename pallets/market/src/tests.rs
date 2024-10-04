use frame_support::*;

use airo_primitives::RequestsUsize;

use crate::{mock::*, *};

fn create_order(consumer: AccountId, model_id: &str, requests_total: RequestsUsize) -> OrderId {
    assert_ok!(AiroMarket::order_create(
        RuntimeOrigin::signed(consumer),
        BoundedVec::try_from(model_id.as_bytes().to_vec()).unwrap(),
        requests_total,
    ));
    CurrentOrderId::<Test>::get()
}

fn create_bid(provider: AccountId, order_id: OrderId, price_per_request: Balance) {
    assert_ok!(AiroMarket::bid_create(
        RuntimeOrigin::signed(provider),
        order_id,
        price_per_request
    ));
}

#[test]
fn can_order() {
    new_test_ext().execute_with(|| {
        let model_id = "test_model";
        let requests_total = 10;

        let order_id = create_order(CONSUMER_1, model_id, requests_total);

        let expected_model_id = BoundedVec::try_from(model_id.as_bytes().to_vec()).unwrap();
        let expected_order =
            OrderDetails::new(CONSUMER_1, expected_model_id.clone(), requests_total);
        assert_eq!(Orders::<Test>::get(order_id), Some(expected_order));
        assert!(ConsumerOrders::<Test>::contains_key(CONSUMER_1, order_id));
        System::assert_last_event(
            Event::OrderCreated { order_id, model_id: expected_model_id }.into(),
        );
    });
}

#[test]
fn fail_order_zero() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AiroMarket::order_create(
                RuntimeOrigin::signed(CONSUMER_1),
                BoundedVec::try_from("model_id".as_bytes().to_vec()).unwrap(),
                0
            ),
            Error::<Test>::OrderInvalid
        );
    });
}

#[test]
fn can_bid() {
    new_test_ext().execute_with(|| {
        let order_id = create_order(CONSUMER_1, "model_id", 1);

        let price = 1000;
        create_bid(PROVIDER_1, order_id, price);

        let expected_bid = BidDetails::new(PROVIDER_1, price);
        assert_eq!(OrderBids::<Test>::get(order_id, PROVIDER_1), Some(expected_bid));
        assert!(ProviderOrders::<Test>::contains_key(PROVIDER_1, order_id));
        System::assert_last_event(
            Event::BidCreated { order_id, provider: PROVIDER_1, price_per_request: price }.into(),
        );
    });
}

#[test]
fn fail_bid_missing_order() {
    new_test_ext().execute_with(|| {
        let order_id = create_order(CONSUMER_1, "model_id", 1);

        assert_noop!(
            AiroMarket::bid_create(RuntimeOrigin::signed(PROVIDER_1), order_id + 1, 1000),
            Error::<Test>::OrderNotFound
        );
    });
}

#[test]
fn fail_bid_same_order() {
    new_test_ext().execute_with(|| {
        let order_id = create_order(CONSUMER_1, "model_id", 1);
        create_bid(PROVIDER_1, order_id, 1000);

        assert_noop!(
            AiroMarket::bid_create(RuntimeOrigin::signed(PROVIDER_1), order_id, 200),
            Error::<Test>::BidAlreadyExists
        );
    });
}

#[test]
fn can_accept_bid() {
    new_test_ext().execute_with(|| {
        let order_id = create_order(CONSUMER_1, "model_id", 5);
        create_bid(PROVIDER_1, order_id, 2000);
        create_bid(PROVIDER_2, order_id, 1000);

        assert_ok!(AiroMarket::bid_accept(RuntimeOrigin::signed(CONSUMER_1), order_id, PROVIDER_2));

        // Check that the order was removed
        assert!(!Orders::<Test>::contains_key(order_id));
        assert!(!OrderBids::<Test>::contains_prefix(order_id));
        assert!(!ProviderOrders::<Test>::contains_prefix(PROVIDER_1));
        assert!(!ProviderOrders::<Test>::contains_prefix(PROVIDER_2));
        assert!(!ConsumerOrders::<Test>::contains_prefix(CONSUMER_1));

        System::assert_last_event(Event::BidAccepted { order_id, provider: PROVIDER_2 }.into());
    });
}

#[test]
fn fail_accept_missing_order() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AiroMarket::bid_accept(RuntimeOrigin::signed(CONSUMER_1), 1, 2),
            Error::<Test>::OrderNotFound
        );
    });
}

#[test]
fn fail_accept_non_owned_order() {
    new_test_ext().execute_with(|| {
        let order_id = create_order(CONSUMER_1, "model_id", 1);
        assert_noop!(
            AiroMarket::bid_accept(RuntimeOrigin::signed(CONSUMER_2), order_id, 2),
            Error::<Test>::OrderInvalid
        );
    });
}

#[test]
fn fail_accept_missing_bid() {
    new_test_ext().execute_with(|| {
        let order_id = create_order(CONSUMER_1, "model_id", 1);
        assert_noop!(
            AiroMarket::bid_accept(RuntimeOrigin::signed(CONSUMER_1), order_id, 2),
            Error::<Test>::BidNotFound
        );
    });
}
