multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ExecuteModule {
    #[endpoint(execute)]
    fn execute(&self) {}
}
