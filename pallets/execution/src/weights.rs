use frame_support::weights::Weight;

/// Weight functions needed for pallet_execution.
pub trait WeightInfo {
    fn request_create() -> Weight;
    fn response_create() -> Weight;
}

/// Weights used for tests only.
impl WeightInfo for () {
    fn request_create() -> Weight {
        Weight::from_parts(10_000_000, 0)
    }

    fn response_create() -> Weight {
        Weight::from_parts(10_000_000, 0)
    }
}
