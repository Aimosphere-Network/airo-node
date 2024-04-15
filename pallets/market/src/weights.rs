use frame_support::weights::Weight;

/// Weight functions needed for pallet_market.
pub trait WeightInfo {
    fn order_create() -> Weight;
    fn bid_create() -> Weight;
    fn bid_accept() -> Weight;
}

/// Weights used for tests only.
impl WeightInfo for () {
    fn order_create() -> Weight {
        Weight::from_parts(10_000_000, 0)
    }

    fn bid_create() -> Weight {
        Weight::from_parts(10_000_000, 0)
    }

    fn bid_accept() -> Weight {
        Weight::from_parts(10_000_000, 0)
    }
}
