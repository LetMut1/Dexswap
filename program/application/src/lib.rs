pub mod dex;
#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;
pub mod error;
pub mod extern_source;
pub mod instruction;
pub mod processor;
pub mod state;
use solana_program::pubkey::Pubkey;
pub const PROGRAM_ID: Pubkey = {
    #[cfg(not(feature = "devnet"))]
    {
        Pubkey::from_str_const("Not implemented yet")
    }
    #[cfg(feature = "devnet")]
    {
        Pubkey::from_str_const("9e7vcyKDvhJePwp1quztrX2dUEjPLchuamWnC6KQAVap")
    }
};
pub const METEORA_V1_PROGRAM_ID: Pubkey = {
    #[cfg(not(feature = "devnet"))]
    {
        Pubkey::from_str_const("Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB")
    }
    #[cfg(feature = "devnet")]
    {
        Pubkey::from_str_const("Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB")
    }
};
pub const METEORA_V1_VAULT_PROGRAM_ID: Pubkey = {
    #[cfg(not(feature = "devnet"))]
    {
        Pubkey::from_str_const("24Uqj9JCLxUeoC3hGfh5W3s9FM9uCHDS2SG3LYwBpyTi")
    }
    #[cfg(feature = "devnet")]
    {
        Pubkey::from_str_const("24Uqj9JCLxUeoC3hGfh5W3s9FM9uCHDS2SG3LYwBpyTi")
    }
};
pub const RAYDIUM_V4_PROGRAM_ID: Pubkey = {
    #[cfg(not(feature = "devnet"))]
    {
        Pubkey::from_str_const("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")
    }
    #[cfg(feature = "devnet")]
    {
        Pubkey::from_str_const("DRaya7Kj3aMWQSy19kSjvmuwq9docCHofyP9kanQGaav")
    }
};
