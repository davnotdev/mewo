pub trait AssetServerBackend {
    fn new() -> Self
    where
        Self: Sized;
    fn load(&mut self, name: &String) -> Result<Vec<u8>, ()>;
    fn should_reload(&mut self, name: &String) -> bool;
}
