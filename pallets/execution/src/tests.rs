use frame_support::{traits::fungible, *};
use sp_core::bounded_vec;
use sp_runtime::TokenError;

use airo_primitives::RequestsUsize;

use crate::{mock::*, *};

fn create_agreement(
    agreement_id: AgreementId,
    consumer: AccountId,
    provider: AccountId,
    price_per_request: Balance,
    requests_total: RequestsUsize,
) {
    assert_ok!(Pallet::<Test>::create_agreement(
        consumer,
        provider,
        agreement_id,
        ModelId::default(),
        price_per_request,
        requests_total,
    ));
}

fn create_request(consumer: AccountId, agreement_id: AgreementId) -> RequestsUsize {
    assert_ok!(Pallet::<Test>::request_create(
        RuntimeOrigin::signed(consumer),
        agreement_id,
        ContentId::default()
    ));

    Agreements::<Test>::get(agreement_id).unwrap().requests_count
}

#[test]
fn can_create_agreement() {
    new_test_ext().execute_with(|| {
        let agreement_id = 1;
        let model_id: ModelId = bounded_vec![1, 2, 3];
        let price_per_request = 100;
        let requests_total = 5;

        assert_ok!(Pallet::<Test>::create_agreement(
            CONSUMER_1,
            PROVIDER_1,
            agreement_id,
            model_id.clone(),
            price_per_request,
            requests_total,
        ));

        let expected_agreement = AgreementDetails::new(
            CONSUMER_1,
            PROVIDER_1,
            model_id,
            price_per_request,
            requests_total,
        );
        assert_eq!(Agreements::<Test>::get(agreement_id), Some(expected_agreement));
        assert!(ConsumerAgreements::<Test>::contains_key(CONSUMER_1, agreement_id));
        assert!(ProviderAgreements::<Test>::contains_key(PROVIDER_1, agreement_id));
        assert_eq!(
            <Balances as fungible::Inspect<_>>::balance(&CONSUMER_1),
            INITIAL_BALANCE - price_per_request * requests_total as Balance
        );
        assert_eq!(
            <Balances as fungible::hold::Inspect<_>>::balance_on_hold(
                &HoldReason::ConsumerPrepayment.into(),
                &CONSUMER_1
            ),
            price_per_request * requests_total as Balance
        );

        System::assert_last_event(Event::AgreementCreated { agreement_id }.into());
    });
}

#[test]
fn fail_create_agreement_no_funds() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Pallet::<Test>::create_agreement(
                CONSUMER_NO_BALANCE,
                PROVIDER_1,
                1,
                ModelId::default(),
                100,
                5,
            ),
            TokenError::FundsUnavailable
        );
    });
}

#[test]
fn can_request() {
    new_test_ext().execute_with(|| {
        let agreement_id = 1;
        create_agreement(agreement_id, CONSUMER_1, PROVIDER_1, 100, 10);

        let content_id = ContentId::random();
        assert_ok!(Pallet::<Test>::request_create(
            RuntimeOrigin::signed(CONSUMER_1),
            agreement_id,
            content_id,
        ));

        assert_eq!(Requests::<Test>::get(agreement_id, 1), Some(content_id));
        assert_eq!(Agreements::<Test>::get(agreement_id).unwrap().requests_count, 1);

        System::assert_last_event(
            Event::RequestCreated { agreement_id, request_index: 1, content_id }.into(),
        );
    });
}

#[test]
fn fail_request_missing_agreement() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Pallet::<Test>::request_create(
                RuntimeOrigin::signed(CONSUMER_1),
                1,
                ContentId::default(),
            ),
            Error::<Test>::AgreementNotFound
        );
    });
}

#[test]
fn fail_request_non_owned_agreement() {
    new_test_ext().execute_with(|| {
        let agreement_id = 1;
        create_agreement(agreement_id, CONSUMER_1, PROVIDER_1, 100, 10);
        assert_noop!(
            Pallet::<Test>::request_create(
                RuntimeOrigin::signed(CONSUMER_2),
                agreement_id,
                ContentId::default(),
            ),
            Error::<Test>::AgreementInvalid
        );
    });
}

#[test]
fn fail_request_exceed_limit() {
    new_test_ext().execute_with(|| {
        let agreement_id = 1;
        create_agreement(agreement_id, CONSUMER_1, PROVIDER_1, 100, 2);
        create_request(CONSUMER_1, agreement_id);
        create_request(CONSUMER_1, agreement_id);

        assert_noop!(
            Pallet::<Test>::request_create(
                RuntimeOrigin::signed(CONSUMER_1),
                agreement_id,
                ContentId::default(),
            ),
            Error::<Test>::RequestNotAllowed
        );
    });
}

#[test]
fn can_respond() {
    new_test_ext().execute_with(|| {
        let agreement_id = 1;
        let price_per_request = 100;
        create_agreement(agreement_id, CONSUMER_1, PROVIDER_1, price_per_request, 10);
        let request_index = create_request(CONSUMER_1, agreement_id);

        let content_id = ContentId::random();
        assert_ok!(Pallet::<Test>::response_create(
            RuntimeOrigin::signed(PROVIDER_1),
            agreement_id,
            request_index,
            content_id,
        ));

        assert_eq!(Responses::<Test>::get(agreement_id, request_index), Some(content_id));
        assert_eq!(<Balances as fungible::Inspect<_>>::balance(&PROVIDER_1), price_per_request);

        System::assert_last_event(
            Event::ResponseCreated { agreement_id, request_index, content_id }.into(),
        );
    });
}

#[test]
fn fail_respond_missing_agreement() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Pallet::<Test>::response_create(
                RuntimeOrigin::signed(PROVIDER_1),
                1,
                1,
                ContentId::default(),
            ),
            Error::<Test>::AgreementNotFound
        );
    });
}

#[test]
fn fail_respond_non_owned_agreement() {
    new_test_ext().execute_with(|| {
        let agreement_id = 1;
        create_agreement(agreement_id, CONSUMER_1, PROVIDER_1, 100, 10);
        let request_index = create_request(CONSUMER_1, agreement_id);

        assert_noop!(
            Pallet::<Test>::response_create(
                RuntimeOrigin::signed(PROVIDER_2),
                agreement_id,
                request_index,
                ContentId::default(),
            ),
            Error::<Test>::AgreementInvalid
        );
    });
}

#[test]
fn fail_respond_missing_request() {
    new_test_ext().execute_with(|| {
        let agreement_id = 1;
        create_agreement(agreement_id, CONSUMER_1, PROVIDER_1, 100, 10);

        assert_noop!(
            Pallet::<Test>::response_create(
                RuntimeOrigin::signed(PROVIDER_1),
                agreement_id,
                1,
                ContentId::default(),
            ),
            Error::<Test>::RequestNotFound
        );
    });
}

#[test]
fn fail_respond_same_request() {
    new_test_ext().execute_with(|| {
        let agreement_id = 1;
        create_agreement(agreement_id, CONSUMER_1, PROVIDER_1, 100, 10);
        let request_index = create_request(CONSUMER_1, agreement_id);

        assert_ok!(Pallet::<Test>::response_create(
            RuntimeOrigin::signed(PROVIDER_1),
            agreement_id,
            request_index,
            ContentId::default(),
        ));
        assert_noop!(
            Pallet::<Test>::response_create(
                RuntimeOrigin::signed(PROVIDER_1),
                agreement_id,
                request_index,
                ContentId::default(),
            ),
            Error::<Test>::ResponseAlreadyExists
        );
    });
}
