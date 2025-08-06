use {
    crate::state::Dex,
    solana_program::{
        instruction::{
            AccountMeta,
            Instruction as Instruction_,
        },
        pubkey::Pubkey,
    },
    std::{
        collections::BTreeSet,
        io::Error,
    },
};
#[repr(C)]
#[derive(Debug, borsh::BorshSerialize, borsh::BorshDeserialize)]
pub enum Instruction {
    Initialize {
        recent_slot: u64,
        lamports_to_treasury: u64,
        w_sol_token_account_pubkey_bump_seed: u8,
        temporary_w_sol_token_account_pubkey_bump_seed: u8,
        self_authority_pubkey_bump_seed: u8,
    },
    DepositFunds {
        lamports_to_treasury: u64,
    },
    WithdrawFunds {
        lamports_from_treasury: u64,
    },
    Swap {
        dexes: Vec<Dex>,
        token_mint: Pubkey,
        quote_mint: Pubkey,
        amount_in: u64,
        min_amount_out: u64,
        token_account_pubkey_bump_seed: u8,
        is_from_quote_to_token: bool,
        with_checks: bool,
    },
}
impl Instruction {
    pub fn initialize(
        program_id: &Pubkey,
        intermediary: &Pubkey,
        intermediary_manager: &Pubkey,
        intermediary_trader: &Pubkey,
        w_sol_token_account: &Pubkey,
        temporary_w_sol_token_account: &Pubkey,
        common_address_lookup_table: &Pubkey,
        self_authority: &Pubkey,
        w_sol_token_mint: &Pubkey,
        system_program_id: &Pubkey,
        rent_program_id: &Pubkey,
        token_program_id: &Pubkey,
        address_lookup_table_program_id: &Pubkey,
        recent_slot: u64,
        lamports_to_treasury: u64,
        w_sol_token_account_pubkey_bump_seed: u8,
        temporary_w_sol_token_account_pubkey_bump_seed: u8,
        self_authority_pubkey_bump_seed: u8,
    ) -> Result<Instruction_, Error> {
        Ok(Instruction_ {
            program_id: *program_id,
            accounts: vec![
                    AccountMeta::new(*intermediary, true),
                    AccountMeta::new_readonly(*intermediary_manager, true),
                    AccountMeta::new_readonly(*intermediary_trader, true),
                    AccountMeta::new(*w_sol_token_account, false),
                    AccountMeta::new(*temporary_w_sol_token_account, false),
                    AccountMeta::new(*common_address_lookup_table, false),
                    AccountMeta::new_readonly(*self_authority, false),
                    AccountMeta::new_readonly(*w_sol_token_mint, false),
                    AccountMeta::new_readonly(*system_program_id, false),
                    AccountMeta::new_readonly(*rent_program_id, false),
                    AccountMeta::new_readonly(*token_program_id, false),
                    AccountMeta::new_readonly(*address_lookup_table_program_id, false),
                ],
            data: borsh::to_vec(&Self::Initialize {
                recent_slot,
                lamports_to_treasury,
                w_sol_token_account_pubkey_bump_seed,
                temporary_w_sol_token_account_pubkey_bump_seed,
                self_authority_pubkey_bump_seed,
            })?,
        })
    }
    pub fn deposit_funds(
        program_id: &Pubkey,
        intermediary: &Pubkey,
        intermediary_manager: &Pubkey,
        w_sol_token_account: &Pubkey,
        system_program_id: &Pubkey,
        token_program_id: &Pubkey,
        lamports_to_treasury: u64,
    ) -> Result<Instruction_, Error> {
        Ok(Instruction_ {
            program_id: *program_id,
            accounts: vec![
                    AccountMeta::new_readonly(*intermediary, false),
                    AccountMeta::new(*intermediary_manager, true),
                    AccountMeta::new(*w_sol_token_account, false),
                    AccountMeta::new_readonly(*system_program_id, false),
                    AccountMeta::new_readonly(*token_program_id, false),
                ],
            data: borsh::to_vec(&Self::DepositFunds {
                lamports_to_treasury,
            })?,
        })
    }
    pub fn withdraw_funds(
        program_id: &Pubkey,
        intermediary: &Pubkey,
        intermediary_manager: &Pubkey,
        w_sol_token_account: &Pubkey,
        temporary_w_sol_token_account: &Pubkey,
        self_authority: &Pubkey,
        w_sol_token_mint: &Pubkey,
        system_program_id: &Pubkey,
        rent_program_id: &Pubkey,
        token_program_id: &Pubkey,
        lamports_from_treasury: u64,
    ) -> Result<Instruction_, Error> {
        Ok(Instruction_ {
            program_id: *program_id,
            accounts: vec![
                    AccountMeta::new_readonly(*intermediary, false),
                    AccountMeta::new(*intermediary_manager, true),
                    AccountMeta::new(*w_sol_token_account, false),
                    AccountMeta::new(*temporary_w_sol_token_account, false),
                    AccountMeta::new_readonly(*self_authority, false),
                    AccountMeta::new_readonly(*w_sol_token_mint, false),
                    AccountMeta::new_readonly(*system_program_id, false),
                    AccountMeta::new_readonly(*rent_program_id, false),
                    AccountMeta::new_readonly(*token_program_id, false),
                ],
            data: borsh::to_vec(&Self::WithdrawFunds {
                lamports_from_treasury,
            })?,
        })
    }
    pub fn swap(
        program_id: &Pubkey,
        intermediary: &Pubkey,
        intermediary_trader: &Pubkey,
        quote_token_account: &Pubkey,
        self_authority: &Pubkey,
        token_account: &Pubkey,
        quote_token_mint: &Pubkey,
        token_mint: &Pubkey,
        system_program_id: &Pubkey,
        rent_program_id: &Pubkey,
        token_program_id: &Pubkey,
        dexes: Vec<Dex_<'_>>,
        amount_in: u64,
        min_amount_out: u64,
        token_account_pubkey_bump_seed: u8,
        is_from_quote_to_token: bool,
        with_checks: bool,
    ) -> Result<Instruction_, Error> {
        if dexes.is_empty() {
            return Err(Error::other("Zero dexes."));
        }
        let mut accounts = vec![
            // For Intermediary
            AccountMeta::new_readonly(*intermediary, false),
            AccountMeta::new(*intermediary_trader, true),
            AccountMeta::new(*quote_token_account, false),
            AccountMeta::new_readonly(*self_authority, false),
            AccountMeta::new(*token_account, false),
            AccountMeta::new_readonly(*quote_token_mint, false),
            AccountMeta::new_readonly(*token_mint, false),
            AccountMeta::new_readonly(*system_program_id, false),
            AccountMeta::new_readonly(*rent_program_id, false),
            AccountMeta::new_readonly(*token_program_id, false),
        ];
        let mut dexes_ = vec![];
        let mut dexes_btree_set = BTreeSet::<Dex>::new();
        '_a: for dex in dexes {
            let dex_ = match dex {
                Dex_::RaydiumV4 {
                    raydium_v4_program_id,
                    clock,
                    token_program_id: token_program_id_,
                    amm_pool,
                    amm_authority,
                    amm_open_orders,
                    amm_coin_vault,
                    amm_pc_vault,
                    market_program_id,
                    market,
                    market_bids,
                    market_asks,
                    market_event_queue,
                    market_coin_vault,
                    market_pc_vault,
                    market_vault_signer,
                } => {
                    accounts.push(AccountMeta::new_readonly(*raydium_v4_program_id, false));
                    accounts.push(AccountMeta::new_readonly(*clock, false));
                    accounts.push(AccountMeta::new_readonly(*token_program_id_, false));
                    accounts.push(AccountMeta::new(*amm_pool, false));
                    accounts.push(AccountMeta::new_readonly(*amm_authority, false));
                    accounts.push(AccountMeta::new(*amm_open_orders, false));
                    accounts.push(AccountMeta::new(*amm_coin_vault, false));
                    accounts.push(AccountMeta::new(*amm_pc_vault, false));
                    accounts.push(AccountMeta::new_readonly(*market_program_id, false));
                    accounts.push(AccountMeta::new(*market, false));
                    accounts.push(AccountMeta::new(*market_bids, false));
                    accounts.push(AccountMeta::new(*market_asks, false));
                    accounts.push(AccountMeta::new(*market_event_queue, false));
                    accounts.push(AccountMeta::new(*market_coin_vault, false));
                    accounts.push(AccountMeta::new(*market_pc_vault, false));
                    accounts.push(AccountMeta::new_readonly(*market_vault_signer, false));
                    Dex::RaydiumV4
                }
                Dex_::MeteoraV1 {
                    meteora_v1_program,
                    clock,
                    pool,
                    a_vault,
                    b_vault,
                    a_token_vault,
                    b_token_vault,
                    a_vault_lp_mint,
                    b_vault_lp_mint,
                    a_vault_lp,
                    b_vault_lp,
                    protocol_token_fee,
                    vault_program,
                    token_program,
                } => {
                    accounts.push(AccountMeta::new_readonly(*meteora_v1_program, false));
                    accounts.push(AccountMeta::new_readonly(*clock, false));
                    accounts.push(AccountMeta::new(*pool, false));
                    accounts.push(AccountMeta::new(*a_vault, false));
                    accounts.push(AccountMeta::new(*b_vault, false));
                    accounts.push(AccountMeta::new(*a_token_vault, false));
                    accounts.push(AccountMeta::new(*b_token_vault, false));
                    accounts.push(AccountMeta::new(*a_vault_lp_mint, false));
                    accounts.push(AccountMeta::new(*b_vault_lp_mint, false));
                    accounts.push(AccountMeta::new(*a_vault_lp, false));
                    accounts.push(AccountMeta::new(*b_vault_lp, false));
                    accounts.push(AccountMeta::new(*protocol_token_fee, false));
                    accounts.push(AccountMeta::new_readonly(*vault_program, false));
                    accounts.push(AccountMeta::new_readonly(*token_program, false));
                    Dex::MeteoraV1
                }
            };
            if !dexes_btree_set.insert(dex_) {
                return Err(Error::other("Repeatable dexes."));
            }
            dexes_.push(dex_);
        }
        Ok(Instruction_ {
            program_id: *program_id,
            accounts,
            data: borsh::to_vec(&Self::Swap {
                dexes: dexes_,
                token_mint: *token_mint,
                quote_mint: *quote_token_mint,
                amount_in,
                min_amount_out,
                token_account_pubkey_bump_seed,
                is_from_quote_to_token,
                with_checks,
            })?,
        })
    }
}
pub enum Dex_<'a> {
    MeteoraV1 {
        meteora_v1_program: &'a Pubkey,
        clock: &'a Pubkey,
        pool: &'a Pubkey,
        a_vault: &'a Pubkey,
        b_vault: &'a Pubkey,
        a_token_vault: &'a Pubkey,
        b_token_vault: &'a Pubkey,
        a_vault_lp_mint: &'a Pubkey,
        b_vault_lp_mint: &'a Pubkey,
        a_vault_lp: &'a Pubkey,
        b_vault_lp: &'a Pubkey,
        protocol_token_fee: &'a Pubkey,
        vault_program: &'a Pubkey,
        token_program: &'a Pubkey,
    },
    RaydiumV4 {
        raydium_v4_program_id: &'a Pubkey,
        clock: &'a Pubkey,
        token_program_id: &'a Pubkey,
        amm_pool: &'a Pubkey,
        amm_authority: &'a Pubkey,
        amm_open_orders: &'a Pubkey,
        amm_coin_vault: &'a Pubkey,
        amm_pc_vault: &'a Pubkey,
        market_program_id: &'a Pubkey,
        market: &'a Pubkey,
        market_bids: &'a Pubkey,
        market_asks: &'a Pubkey,
        market_event_queue: &'a Pubkey,
        market_coin_vault: &'a Pubkey,
        market_pc_vault: &'a Pubkey,
        market_vault_signer: &'a Pubkey,
    },
}
