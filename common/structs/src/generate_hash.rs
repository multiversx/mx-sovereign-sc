use multiversx_sc::{
    api::{CryptoApi, CryptoApiImpl, SHA256_RESULT_LEN},
    codec::TopEncode,
    types::{ManagedBuffer, ManagedByteArray, ManagedType},
};

pub trait GenerateHash<A: CryptoApi>
where
    Self: TopEncode,
{
    fn generate_hash(&self) -> ManagedBuffer<A> {
        let mut serialized_data = ManagedBuffer::<A>::new();

        if self.top_encode(&mut serialized_data).is_err() {
            return ManagedBuffer::new();
        }

        unsafe {
            let result: ManagedByteArray<A, SHA256_RESULT_LEN> = ManagedByteArray::new_uninit();
            A::crypto_api_impl().sha256_managed(result.get_handle(), serialized_data.get_handle());

            result.as_managed_buffer().clone()
        }
    }
}
