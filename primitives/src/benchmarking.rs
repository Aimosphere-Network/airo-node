pub trait ModelFactory<ModelId> {
    fn get_model_id() -> ModelId;
}

pub trait ContentFactory<ContentId> {
    fn get_content_id() -> ContentId;
}
