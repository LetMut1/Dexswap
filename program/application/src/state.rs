use {
    crate::{
        METEORA_V1_PROGRAM_ID,
        METEORA_V1_VAULT_PROGRAM_ID,
        PROGRAM_ID,
        RAYDIUM_V4_PROGRAM_ID,
    },
    bytemuck::*,
    solana_program::pubkey::{
        Pubkey,
        PubkeyError,
    },
};
// Addresses:
// - intermediary
// - w_sol_token_account
// - self_authority
pub const QUANTITY_OF_MUCH_USED_DYNAMIC_ACCOUNTS: u8 = {
    const QUANTITY_OF_MUCH_USED_DYNAMIC_ACCOUNTS_: u8 = 3;
    static_assertions::const_assert!(
        QUANTITY_OF_MUCH_USED_DYNAMIC_ACCOUNTS_ > 0
    );
    QUANTITY_OF_MUCH_USED_DYNAMIC_ACCOUNTS_
};
pub const MUCH_USED_STATIC_ACCOUNTS: [Pubkey; 8] = {
    const MUCH_USED_STATIC_ACCOUNTS_: [Pubkey; 8] = [
        spl_token::native_mint::ID,
        solana_program::system_program::ID,
        solana_program::sysvar::rent::ID,
        spl_token::ID,
        solana_program::sysvar::clock::ID,
        METEORA_V1_PROGRAM_ID,
        METEORA_V1_VAULT_PROGRAM_ID,
        RAYDIUM_V4_PROGRAM_ID,
    ];
    static_assertions::const_assert!(
        MUCH_USED_STATIC_ACCOUNTS_.len() <= u8::MAX as usize
    );
    MUCH_USED_STATIC_ACCOUNTS_
};
const _: () = {
    static_assertions::const_assert!(
        MUCH_USED_STATIC_ACCOUNTS.len() + QUANTITY_OF_MUCH_USED_DYNAMIC_ACCOUNTS as usize <= u8::MAX as usize
    );
};
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Intermediary {
    // Manages data in the base Self-state.
    //
    // Self-state owner should have Keypair for this account.
    pub manager: Pubkey,
    // Performs token exchange in any methods where exchange occurs through the CPI.
    //
    // Self-state owner should have Keypair for this account.
    pub trader: Pubkey,
    // A token account for storing Wsol liquidity for exchange. It is necessary so that
    // with each swap you do not have to create, initialize, and transfer tokens.
    // Reduces the amount of transaction costs during automatic trading.
    //
    // Owner - self.self_authority. That is, all manipulations with reducing the token amount
    // on the account are carried out through a this contract.
    //
    // Should be PDA-derived.
    pub w_sol_token_account: Pubkey,
    // A token account used as a temporary one in method for withdrawing Wsols from any self WSol liquidity.
    //
    // Owner - self.self_authority. That is, all manipulations with reducing the token amount
    // on the account are carried out through a this contract.
    //
    // Should be PDA-derived.
    pub temporary_w_sol_token_account: Pubkey,
    // An ALT-account for storing a basic set of accounts for our needs.
    pub common_address_lookup_table: Pubkey,
    // Signs PDA accounts to implement the "contract-owner" concept.
    //
    // Should be PDA-derived.
    pub self_authority: Pubkey,
    pub w_sol_token_account_pubkey_bump_seed: u8,
    pub temporary_w_sol_token_account_pubkey_bump_seed: u8,
    pub self_authority_pubkey_bump_seed: u8,
    // State of Self-state
    //
    // 0 -> Not,
    // 1 - Yes,
    is_initialized: u8,
}
impl Intermediary {
    pub fn new(
        manager: Pubkey,
        trader: Pubkey,
        w_sol_token_account: Pubkey,
        temporary_w_sol_token_account: Pubkey,
        common_address_lookup_table: Pubkey,
        self_authority: Pubkey,
        w_sol_token_account_pubkey_bump_seed: u8,
        temporary_w_sol_token_account_pubkey_bump_seed: u8,
        self_authority_pubkey_bump_seed: u8,
    ) -> Self {
        Self {
            manager,
            trader,
            w_sol_token_account,
            temporary_w_sol_token_account,
            common_address_lookup_table,
            self_authority,
            w_sol_token_account_pubkey_bump_seed,
            temporary_w_sol_token_account_pubkey_bump_seed,
            self_authority_pubkey_bump_seed,
            is_initialized: 1,
        }
    }
    pub fn is_initialized(&self) -> bool {
        self.is_initialized == 1
    }
}
unsafe impl Pod for Intermediary {}
unsafe impl Zeroable for Intermediary {}
pub struct PdaResolver;
impl PdaResolver {
    const TOKEN_ACCOUNT_SEED: &'static str = "tokenaccount";
    const TEMPORARY_W_SOL_TOKEN_ACCOUNT_SEED: &'static str = "temporarywsoltokenaccount";
    const SELF_AUTHORITY_SEED: &'static str = "selfauthority";
    pub fn token_account_get_seeds<'a>(intermediary: &'a Pubkey, token_mint: &'a Pubkey, bump_seed: &'a [u8]) -> [&'a [u8]; 5] {
        [
            PROGRAM_ID.as_ref(),
            intermediary.as_ref(),
            token_mint.as_ref(),
            Self::TOKEN_ACCOUNT_SEED.as_bytes(),
            bump_seed,
        ]
    }
    pub fn token_account_create(intermediary: &Pubkey, token_mint: &Pubkey, bump_seed: u8) -> Result<Pubkey, PubkeyError> {
        Pubkey::create_program_address(Self::token_account_get_seeds(intermediary, token_mint, [bump_seed].as_slice()).as_slice(), &PROGRAM_ID)
    }
    pub fn temporary_w_sol_token_account_find(intermediary: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            [
                PROGRAM_ID.as_ref(),
                intermediary.as_ref(),
                Self::TEMPORARY_W_SOL_TOKEN_ACCOUNT_SEED.as_bytes(),
            ]
            .as_slice(),
            &PROGRAM_ID,
        )
    }
    pub fn temporary_w_sol_token_account_get_seeds<'a>(intermediary: &'a Pubkey, bump_seed: &'a [u8]) -> [&'a [u8]; 4] {
        [
            PROGRAM_ID.as_ref(),
            intermediary.as_ref(),
            Self::TEMPORARY_W_SOL_TOKEN_ACCOUNT_SEED.as_bytes(),
            bump_seed,
        ]
    }
    pub fn temporary_w_sol_token_account_create(intermediary: &Pubkey, bump_seed: u8) -> Result<Pubkey, PubkeyError> {
        Pubkey::create_program_address(Self::temporary_w_sol_token_account_get_seeds(intermediary, [bump_seed].as_slice()).as_slice(), &PROGRAM_ID)
    }
    pub fn self_authority_find(intermediary: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            [
                PROGRAM_ID.as_ref(),
                intermediary.as_ref(),
                Self::SELF_AUTHORITY_SEED.as_bytes(),
            ]
            .as_slice(),
            &PROGRAM_ID,
        )
    }
    pub fn self_authority_get_seeds<'a>(intermediary: &'a Pubkey, bump_seed: &'a [u8]) -> [&'a [u8]; 4] {
        [
            PROGRAM_ID.as_ref(),
            intermediary.as_ref(),
            Self::SELF_AUTHORITY_SEED.as_bytes(),
            bump_seed,
        ]
    }
    pub fn self_authority_create(intermediary: &Pubkey, bump_seed: u8) -> Result<Pubkey, PubkeyError> {
        Pubkey::create_program_address(Self::self_authority_get_seeds(intermediary, [bump_seed].as_slice()).as_slice(), &PROGRAM_ID)
    }
    pub fn token_account_find(intermediary: &Pubkey, token_mint: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            [
                PROGRAM_ID.as_ref(),
                intermediary.as_ref(),
                token_mint.as_ref(),
                Self::TOKEN_ACCOUNT_SEED.as_bytes(),
            ]
            .as_slice(),
            &PROGRAM_ID,
        )
    }
}
#[repr(C)]
#[derive(Debug, borsh::BorshSerialize, borsh::BorshDeserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dex {
    MeteoraV1,
    RaydiumV4,
}
impl Dex {
    pub fn to_str(&self) -> &'static str {
        match *self {
            Self::MeteoraV1 => "MeteoraV1",
            Self::RaydiumV4 => "RaydiumV4",
        }
    }
}
