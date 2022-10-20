pub trait AssetConverter {
    fn convert(data: Vec<u8>) -> Self
    where
        Self: Sized;
}
