use frame_support::*;

use crate::{mock::*, *};

fn create_order(consumer: u64, model_id: &str, requests_total: u32) -> u32 {
    assert_ok!(AIMarket::order_create(
        RuntimeOrigin::signed(consumer),
        BoundedVec::try_from(model_id.as_bytes().to_vec()).unwrap(),
        requests_total,
    ));
    CurrentOrderId::<Test>::get()
}

fn create_bid(provider: u64, order_id: u32, price_per_request: u64) {
    assert_ok!(AIMarket::bid_create(RuntimeOrigin::signed(provider), order_id, price_per_request));
}

#[test]
fn can_order() {
    new_test_ext().execute_with(|| {
        // Go past genesis block so events get deposited
        System::set_block_number(1);

        let consumer = 111;
        let model_id = "test_model";
        let requests_total = 10;

        let order_id = create_order(consumer, model_id, requests_total);

        let expected_model_id = BoundedVec::try_from(model_id.as_bytes().to_vec()).unwrap();
        let expected_order = OrderDetails::new(consumer, expected_model_id.clone(), requests_total);
        assert_eq!(Orders::<Test>::get(order_id), Some(expected_order));
        assert!(ConsumerOrders::<Test>::contains_key(consumer, order_id));
        System::assert_last_event(
            Event::OrderCreated { order_id, model_id: expected_model_id }.into(),
        );
    });
}

#[test]
fn fail_order_zero() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AIMarket::order_create(
                RuntimeOrigin::signed(1),
                BoundedVec::try_from("model_id".as_bytes().to_vec()).unwrap(),
                0
            ),
            Error::<Test>::OrderInvalid
        );
    });
}

#[test]
fn fail_order_default_model_id() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AIMarket::order_create(
                RuntimeOrigin::signed(1),
                BoundedVec::try_from("".as_bytes().to_vec()).unwrap(),
                1
            ),
            Error::<Test>::OrderInvalid
        );
    });
}

#[test]
fn can_bid() {
    new_test_ext().execute_with(|| {
        // Go past genesis block so events get deposited
        System::set_block_number(1);

        let order_id = create_order(1, "model_id", 1);

        let provider = 2;
        let price = 1000;
        create_bid(provider, order_id, price);

        let expected_bid = Bid::new(provider, price);
        assert_eq!(OrderBids::<Test>::get(order_id, provider), Some(expected_bid));
        assert!(ProviderOrders::<Test>::contains_key(provider, order_id));
        System::assert_last_event(
            Event::BidCreated { order_id, provider, price_per_request: price }.into(),
        );
    });
}

#[test]
fn fail_bid_missing_order() {
    new_test_ext().execute_with(|| {
        let order_id = create_order(1, "model_id", 1);

        assert_noop!(
            AIMarket::bid_create(RuntimeOrigin::signed(1), order_id + 1, 1000),
            Error::<Test>::OrderNotFound
        );
    });
}

#[test]
fn fail_bid_same_order() {
    new_test_ext().execute_with(|| {
        let order_id = create_order(1, "model_id", 1);
        let provider = 2;
        create_bid(provider, order_id, 1000);

        assert_noop!(
            AIMarket::bid_create(RuntimeOrigin::signed(provider), order_id, 200),
            Error::<Test>::BidAlreadyExists
        );
    });
}

#[test]
fn can_accept_bid() {
    new_test_ext().execute_with(|| {
        // Go past genesis block so events get deposited
        System::set_block_number(1);

        let consumer = 1;
        let order_id = create_order(consumer, "model_id", 5);
        let provider_1 = 10;
        create_bid(provider_1, order_id, 2000);
        let provider_2 = 20;
        create_bid(provider_2, order_id, 1000);

        assert_ok!(AIMarket::bid_accept(RuntimeOrigin::signed(consumer), order_id, provider_2));

        // TODO. Check execution flow is started

        // Check that the order was removed
        assert!(!Orders::<Test>::contains_key(order_id));
        assert!(!OrderBids::<Test>::contains_prefix(order_id));
        assert!(!ProviderOrders::<Test>::contains_prefix(provider_1));
        assert!(!ProviderOrders::<Test>::contains_prefix(provider_2));
        assert!(!ConsumerOrders::<Test>::contains_prefix(consumer));

        System::assert_last_event(Event::BidAccepted { order_id, provider: provider_2 }.into());
    });
}

#[test]
fn fail_accept_missing_order() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            AIMarket::bid_accept(RuntimeOrigin::signed(1), 1, 2),
            Error::<Test>::OrderNotFound
        );
    });
}

#[test]
fn fail_accept_non_owned_order() {
    new_test_ext().execute_with(|| {
        let owner = 1;
        let order_id = create_order(owner, "model_id", 1);
        let non_owner = 2;
        assert_noop!(
            AIMarket::bid_accept(RuntimeOrigin::signed(non_owner), order_id, 2),
            Error::<Test>::OrderInvalid
        );
    });
}

#[test]
fn fail_accept_missing_bid() {
    new_test_ext().execute_with(|| {
        let consumer = 1;
        let order_id = create_order(consumer, "model_id", 1);
        assert_noop!(
            AIMarket::bid_accept(RuntimeOrigin::signed(consumer), order_id, 2),
            Error::<Test>::BidNotFound
        );
    });
}
