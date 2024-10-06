pub trait RoyaltyResolver {
    type AccountId;
    type Balance;
    type ModelId;

    fn get_royalty(model_id: &Self::ModelId) -> Option<(Self::AccountId, Self::Balance)>;
}
