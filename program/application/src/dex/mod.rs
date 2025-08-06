pub mod meteora_v1;
pub mod raydium_v4;
use {
    crate::state::Intermediary,
    solana_program::{
        account_info::AccountInfo,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};
pub trait Dex<'a, 'b, 'c> {
    fn get_swap_accounts_quantity(&'a self) -> usize;
    fn do_swap_calculation(&'a self, base_data: &'a BaseData<'b, 'c>, first_account_index: usize) -> Result<Option<SwapCalculationResult>, ProgramError>;
    fn do_swap(&'a self, base_data: &'a BaseData<'b, 'c>, first_account_index: usize) -> Result<(), ProgramError>;
}
pub struct SwapCalculationResult {
    pub pool: Pubkey,
    pub amount_in_fee: u64,
    pub amount_out: u64,
}
pub struct BaseData<'a, 'b> {
    pub accounts: &'a [AccountInfo<'b>],
    pub intermediary: &'a AccountInfo<'b>,
    pub quote_token_account: &'a AccountInfo<'b>,
    pub token_account: &'a AccountInfo<'b>,
    pub self_authority: &'a AccountInfo<'b>,
    pub intermediary_: &'a Intermediary,
    pub token_mint: &'a Pubkey,
    pub quote_mint: &'a Pubkey,
    pub amount_in: u64,
    pub min_amount_out: u64,
    pub is_from_quote_to_token: bool,
    pub with_checks: bool,
}
