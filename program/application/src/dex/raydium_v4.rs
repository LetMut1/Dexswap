use {
    super::{
        BaseData,
        Dex,
        SwapCalculationResult,
    },
    crate::{
        RAYDIUM_V4_PROGRAM_ID,
        error::Error,
        extern_source::{
            CheckedCeilDiv,
            raydium_v4::{
                AmmInfo,
                AmmStatus,
                SwapDirection,
                U128,
            },
        },
        state::PdaResolver,
    },
    solana_program::{
        clock::Clock,
        instruction::{
            AccountMeta,
            Instruction,
        },
        program_error::ProgramError,
        pubkey::Pubkey,
        sysvar::Sysvar,
    },
};
pub struct RaydiumV4;
impl RaydiumV4 {
    // https://github.com/raydium-io/raydium-amm/blob/2748852a7981c2b6909e07e10b1325669fbb9195/program/src/instruction.rs#L1045
    pub fn create_swap_instruction(
        program_id: &Pubkey,
        token_program: &Pubkey,
        amm_pool: &Pubkey,
        amm_authority: &Pubkey,
        amm_open_orders: &Pubkey,
        amm_coin_vault: &Pubkey,
        amm_pc_vault: &Pubkey,
        market_program: &Pubkey,
        market: &Pubkey,
        market_bids: &Pubkey,
        market_asks: &Pubkey,
        market_event_queue: &Pubkey,
        market_coin_vault: &Pubkey,
        market_pc_vault: &Pubkey,
        market_vault_signer: &Pubkey,
        user_token_source: &Pubkey,
        user_token_destination: &Pubkey,
        user_source_owner: &Pubkey,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Instruction {
        let mut data = Vec::<u8>::with_capacity(17);
        // https://github.com/raydium-io/raydium-amm/blob/2748852a7981c2b6909e07e10b1325669fbb9195/program/src/instruction.rs#L758
        data.push(9);
        data.extend(amount_in.to_le_bytes());
        data.extend(minimum_amount_out.to_le_bytes());
        Instruction {
            program_id: *program_id,
            accounts: vec![
                AccountMeta::new_readonly(*token_program, false),
                AccountMeta::new(*amm_pool, false),
                AccountMeta::new_readonly(*amm_authority, false),
                AccountMeta::new(*amm_open_orders, false),
                AccountMeta::new(*amm_coin_vault, false),
                AccountMeta::new(*amm_pc_vault, false),
                AccountMeta::new_readonly(*market_program, false),
                AccountMeta::new(*market, false),
                AccountMeta::new(*market_bids, false),
                AccountMeta::new(*market_asks, false),
                AccountMeta::new(*market_event_queue, false),
                AccountMeta::new(*market_coin_vault, false),
                AccountMeta::new(*market_pc_vault, false),
                AccountMeta::new_readonly(*market_vault_signer, false),
                AccountMeta::new(*user_token_source, false),
                AccountMeta::new(*user_token_destination, false),
                AccountMeta::new_readonly(*user_source_owner, true),
            ],
            data,
        }
    }
}
impl<'a, 'b, 'c> Dex<'a, 'b, 'c> for RaydiumV4 {
    fn get_swap_accounts_quantity(&'a self) -> usize {
        16
    }
    fn do_swap_calculation(&'a self, base_data: &'a BaseData<'b, 'c>, first_account_index: usize) -> Result<Option<SwapCalculationResult>, ProgramError> {
        let account_info_iter = &mut base_data.accounts.iter().skip(first_account_index).take(self.get_swap_accounts_quantity());
        let raydium_v4_program_id = solana_program::account_info::next_account_info(account_info_iter)?;
        let clock = solana_program::account_info::next_account_info(account_info_iter)?;
        let token_program = solana_program::account_info::next_account_info(account_info_iter)?;
        let amm_pool = solana_program::account_info::next_account_info(account_info_iter)?;
        let amm_authority = solana_program::account_info::next_account_info(account_info_iter)?;
        let amm_open_orders = solana_program::account_info::next_account_info(account_info_iter)?;
        let amm_coin_vault = solana_program::account_info::next_account_info(account_info_iter)?;
        let amm_pc_vault = solana_program::account_info::next_account_info(account_info_iter)?;
        let _market_program = solana_program::account_info::next_account_info(account_info_iter)?;
        let market = solana_program::account_info::next_account_info(account_info_iter)?;
        let market_bids = solana_program::account_info::next_account_info(account_info_iter)?;
        let market_asks = solana_program::account_info::next_account_info(account_info_iter)?;
        let market_event_queue = solana_program::account_info::next_account_info(account_info_iter)?;
        let market_coin_vault = solana_program::account_info::next_account_info(account_info_iter)?;
        let market_pc_vault = solana_program::account_info::next_account_info(account_info_iter)?;
        let _market_vault_signer = solana_program::account_info::next_account_info(account_info_iter)?;
        if *raydium_v4_program_id.key != RAYDIUM_V4_PROGRAM_ID {
            return Err(Error::InvalidAccountPubkey.into());
        }
        if base_data.with_checks
            && (!amm_pool.is_writable
                || !amm_open_orders.is_writable
                || !amm_coin_vault.is_writable
                || !amm_pc_vault.is_writable
                || !market.is_writable
                || !market_bids.is_writable
                || !market_asks.is_writable
                || !market_event_queue.is_writable
                || !market_coin_vault.is_writable
                || !market_pc_vault.is_writable)
        {
            return Err(Error::InvalidAccountConfigurationFlags.into());
        }
        // This is a slightly modified selective code from https://github.com/raydium-io/raydium-amm/blob/2748852a7981c2b6909e07e10b1325669fbb9195/program/src/processor.rs#L2210
        // that calculates the swap possibility.
        //
        // In ideal case all structures here should be deserealized from accounts in zero-copy context.
        let swap_calculation_result = {
            if *token_program.key != spl_token::ID {
                return Err(Error::InvalidSplTokenProgram.into());
            }
            let mut amm_info = match AmmInfo::load_mut_checked(amm_pool, raydium_v4_program_id.key) {
                Ok(amm_info_) => amm_info_,
                Err(error) => {
                    match error {
                        ProgramError::Custom(code) => {
                            if code == Error::InvalidStatus as u32 {
                                return Ok(None);
                            } else {
                                return Err(error);
                            }
                        }
                        _ => return Err(error),
                    }
                }
            };
            let amm_coin_vault = crate::extern_source::raydium_v4::unpack_token_account(amm_coin_vault, token_program.key)?;
            let amm_pc_vault = crate::extern_source::raydium_v4::unpack_token_account(amm_pc_vault, token_program.key)?;
            if !AmmStatus::from_u64(amm_info.status).swap_permission() {
                let clock_ = Clock::from_account_info(clock)?;
                if amm_info.status == AmmStatus::OrderBookOnly.into_u64() && (clock_.unix_timestamp as u64) >= amm_info.state_data.orderbook_to_init_time {
                    amm_info.status = AmmStatus::Initialized.into_u64();
                } else {
                    return Ok(None);
                }
            } else if amm_info.status == AmmStatus::WaitingTrade.into_u64() {
                let clock_ = Clock::from_account_info(clock)?;
                if (clock_.unix_timestamp as u64) < amm_info.state_data.pool_open_time {
                    return Ok(None);
                } else {
                    amm_info.status = AmmStatus::SwapOnly.into_u64();
                }
            }
            let (total_pc_without_take_pnl, total_coin_without_take_pnl) = if AmmStatus::from_u64(amm_info.status).orderbook_permission() {
                let (market_state, open_orders) = crate::extern_source::raydium_v4::load_serum_market_order(market, amm_open_orders, amm_authority, &amm_info, false)?;
                // Calculator::calc_total_without_take_pnl() writes logs.
                // We don't need that, so we use a method that does the same thing but doesn't write logs.
                crate::extern_source::raydium_v4::calc_total_without_take_pnl(
                    amm_pc_vault.amount,
                    amm_coin_vault.amount,
                    &open_orders,
                    &amm_info,
                    &market_state,
                    market_event_queue,
                    amm_open_orders,
                )?
            } else {
                crate::extern_source::raydium_v4::calc_total_without_take_pnl_no_orderbook(amm_pc_vault.amount, amm_coin_vault.amount, &amm_info)?
            };
            let user_source = crate::extern_source::raydium_v4::unpack_token_account(base_data.quote_token_account, token_program.key)?;
            let user_destination = crate::extern_source::raydium_v4::unpack_token_account(base_data.token_account, token_program.key)?;
            let swap_direction = if user_source.mint == amm_coin_vault.mint && user_destination.mint == amm_pc_vault.mint {
                SwapDirection::Coin2PC
            } else if user_source.mint == amm_pc_vault.mint && user_destination.mint == amm_coin_vault.mint {
                SwapDirection::PC2Coin
            } else {
                return Err(Error::InvalidUserToken.into());
            };
            let swap_fee = U128::from(base_data.amount_in)
                .checked_mul(amm_info.fees.swap_fee_numerator.into())
                .ok_or(ProgramError::ArithmeticOverflow)?
                .checked_ceil_div(amm_info.fees.swap_fee_denominator.into())
                .ok_or(ProgramError::ArithmeticOverflow)?
                .0;
            let swap_in_after_deduct_fee = U128::from(base_data.amount_in).checked_sub(swap_fee).ok_or(ProgramError::ArithmeticOverflow)?;
            let amount_out_ = crate::extern_source::raydium_v4::swap_token_amount_base_in(
                swap_in_after_deduct_fee,
                total_pc_without_take_pnl.into(),
                total_coin_without_take_pnl.into(),
                swap_direction,
            )
            .as_u64();
            match swap_direction {
                SwapDirection::Coin2PC => {
                    if amount_out_ >= total_pc_without_take_pnl {
                        return Ok(None);
                    }
                }
                SwapDirection::PC2Coin => {
                    if amount_out_ >= total_coin_without_take_pnl {
                        return Ok(None);
                    }
                }
            };
            SwapCalculationResult {
                pool: *amm_pool.key,
                amount_in_fee: swap_fee.try_into().map_err(|_| ProgramError::ArithmeticOverflow)?,
                amount_out: amount_out_,
            }
        };
        Ok(Some(swap_calculation_result))
    }
    fn do_swap(&'a self, base_data: &'a BaseData<'b, 'c>, first_account_index: usize) -> Result<(), ProgramError> {
        let account_info_iter = &mut base_data.accounts.iter().skip(first_account_index).take(self.get_swap_accounts_quantity());
        let raydium_v4_program_id = solana_program::account_info::next_account_info(account_info_iter)?;
        let _clock = solana_program::account_info::next_account_info(account_info_iter)?;
        let token_program = solana_program::account_info::next_account_info(account_info_iter)?;
        let amm_pool = solana_program::account_info::next_account_info(account_info_iter)?;
        let amm_authority = solana_program::account_info::next_account_info(account_info_iter)?;
        let amm_open_orders = solana_program::account_info::next_account_info(account_info_iter)?;
        let amm_coin_vault = solana_program::account_info::next_account_info(account_info_iter)?;
        let amm_pc_vault = solana_program::account_info::next_account_info(account_info_iter)?;
        let market_program = solana_program::account_info::next_account_info(account_info_iter)?;
        let market = solana_program::account_info::next_account_info(account_info_iter)?;
        let market_bids = solana_program::account_info::next_account_info(account_info_iter)?;
        let market_asks = solana_program::account_info::next_account_info(account_info_iter)?;
        let market_event_queue = solana_program::account_info::next_account_info(account_info_iter)?;
        let market_coin_vault = solana_program::account_info::next_account_info(account_info_iter)?;
        let market_pc_vault = solana_program::account_info::next_account_info(account_info_iter)?;
        let market_vault_signer = solana_program::account_info::next_account_info(account_info_iter)?;
        if base_data.is_from_quote_to_token {
            solana_program::program::invoke_signed(
                &Self::create_swap_instruction(
                    raydium_v4_program_id.key,
                    token_program.key,
                    amm_pool.key,
                    amm_authority.key,
                    amm_open_orders.key,
                    amm_coin_vault.key,
                    amm_pc_vault.key,
                    market_program.key,
                    market.key,
                    market_bids.key,
                    market_asks.key,
                    market_event_queue.key,
                    market_coin_vault.key,
                    market_pc_vault.key,
                    market_vault_signer.key,
                    base_data.quote_token_account.key,
                    base_data.token_account.key,
                    base_data.self_authority.key,
                    base_data.amount_in,
                    base_data.min_amount_out,
                ),
                vec![
                    token_program.clone(),
                    amm_pool.clone(),
                    amm_authority.clone(),
                    amm_open_orders.clone(),
                    amm_coin_vault.clone(),
                    amm_pc_vault.clone(),
                    market_program.clone(),
                    market.clone(),
                    market_bids.clone(),
                    market_asks.clone(),
                    market_event_queue.clone(),
                    market_coin_vault.clone(),
                    market_pc_vault.clone(),
                    market_vault_signer.clone(),
                    base_data.quote_token_account.clone(),
                    base_data.token_account.clone(),
                    base_data.self_authority.clone(),
                ]
                .as_slice(),
                [PdaResolver::self_authority_get_seeds(base_data.intermediary.key, [base_data.intermediary_.self_authority_pubkey_bump_seed].as_slice()).as_slice()].as_slice(),
            )
        } else {
            solana_program::program::invoke_signed(
                &Self::create_swap_instruction(
                    raydium_v4_program_id.key,
                    token_program.key,
                    amm_pool.key,
                    amm_authority.key,
                    amm_open_orders.key,
                    amm_coin_vault.key,
                    amm_pc_vault.key,
                    market_program.key,
                    market.key,
                    market_bids.key,
                    market_asks.key,
                    market_event_queue.key,
                    market_coin_vault.key,
                    market_pc_vault.key,
                    market_vault_signer.key,
                    base_data.token_account.key,
                    base_data.quote_token_account.key,
                    base_data.self_authority.key,
                    base_data.amount_in,
                    base_data.min_amount_out,
                ),
                vec![
                    token_program.clone(),
                    amm_pool.clone(),
                    amm_authority.clone(),
                    amm_open_orders.clone(),
                    amm_coin_vault.clone(),
                    amm_pc_vault.clone(),
                    market_program.clone(),
                    market.clone(),
                    market_bids.clone(),
                    market_asks.clone(),
                    market_event_queue.clone(),
                    market_coin_vault.clone(),
                    market_pc_vault.clone(),
                    market_vault_signer.clone(),
                    base_data.token_account.clone(),
                    base_data.quote_token_account.clone(),
                    base_data.self_authority.clone(),
                ]
                .as_slice(),
                [PdaResolver::self_authority_get_seeds(base_data.intermediary.key, [base_data.intermediary_.self_authority_pubkey_bump_seed].as_slice()).as_slice()].as_slice(),
            )
        }
    }
}
