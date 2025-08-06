use {
    super::{
        BaseData,
        Dex,
        SwapCalculationResult,
    },
    crate::{
        METEORA_V1_PROGRAM_ID,
        METEORA_V1_VAULT_PROGRAM_ID,
        error::Error,
        extern_source::meteora_v1::{
            ActivationType,
            ConstantProduct,
            CurveType,
            Pool,
            SwapCurve,
            SwapResult,
            TradeDirection,
            Vault,
        },
        state::PdaResolver,
    },
    solana_program::{
        account_info::AccountInfo,
        clock::Clock,
        instruction::{
            AccountMeta,
            Instruction,
        },
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
        sysvar::Sysvar,
    },
    spl_token::state::{
        Account,
        Mint,
    },
};
const FIRST_BYTE_INDEX_AFTER_ANCHOR_DEFAULT_LENGTH_DISCRIMINATOR: usize = 8;
pub struct MeteoraV1;
impl MeteoraV1 {
    pub fn create_swap_instruction(
        program_id: &Pubkey,
        pool: &Pubkey,
        user_source_token: &Pubkey,
        user_destination_token: &Pubkey,
        a_vault: &Pubkey,
        b_vault: &Pubkey,
        a_token_vault: &Pubkey,
        b_token_vault: &Pubkey,
        a_vault_lp_mint: &Pubkey,
        b_vault_lp_mint: &Pubkey,
        a_vault_lp: &Pubkey,
        b_vault_lp: &Pubkey,
        protocol_token_fee: &Pubkey,
        user: &Pubkey,
        vault_program: &Pubkey,
        token_program: &Pubkey,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Instruction {
        let mut data = Vec::<u8>::with_capacity(24);
        const DISCRIMINATOR: [u8; 8] = [
            248,
            198,
            158,
            145,
            225,
            117,
            135,
            200,
        ];
        data.extend(DISCRIMINATOR);
        data.extend(amount_in.to_le_bytes());
        data.extend(minimum_amount_out.to_le_bytes());
        Instruction {
            program_id: *program_id,
            accounts: vec![
                // https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-amm/src/instructions/swap.rs#L5
                AccountMeta::new(*pool, false),
                AccountMeta::new(*user_source_token, false),
                AccountMeta::new(*user_destination_token, false),
                AccountMeta::new(*a_vault, false),
                AccountMeta::new(*b_vault, false),
                AccountMeta::new(*a_token_vault, false),
                AccountMeta::new(*b_token_vault, false),
                AccountMeta::new(*a_vault_lp_mint, false),
                AccountMeta::new(*b_vault_lp_mint, false),
                AccountMeta::new(*a_vault_lp, false),
                AccountMeta::new(*b_vault_lp, false),
                AccountMeta::new(*protocol_token_fee, false),
                AccountMeta::new_readonly(*user, true),
                AccountMeta::new_readonly(*vault_program, false),
                AccountMeta::new_readonly(*token_program, false),
            ],
            data,
        }
    }
}
impl<'a, 'b, 'c> Dex<'a, 'b, 'c> for MeteoraV1 {
    fn get_swap_accounts_quantity(&'a self) -> usize {
        14
    }
    fn do_swap_calculation(&'a self, base_data: &'a BaseData<'b, 'c>, first_account_index: usize) -> Result<Option<SwapCalculationResult>, ProgramError> {
        let account_info_iter = &mut base_data.accounts.iter().skip(first_account_index).take(self.get_swap_accounts_quantity());
        let meteora_v1_program = solana_program::account_info::next_account_info(account_info_iter)?;
        let clock = solana_program::account_info::next_account_info(account_info_iter)?;
        let pool = solana_program::account_info::next_account_info(account_info_iter)?;
        let a_vault = solana_program::account_info::next_account_info(account_info_iter)?;
        let b_vault = solana_program::account_info::next_account_info(account_info_iter)?;
        let a_token_vault = solana_program::account_info::next_account_info(account_info_iter)?;
        let b_token_vault = solana_program::account_info::next_account_info(account_info_iter)?;
        let a_vault_lp_mint = solana_program::account_info::next_account_info(account_info_iter)?;
        let b_vault_lp_mint = solana_program::account_info::next_account_info(account_info_iter)?;
        let a_vault_lp = solana_program::account_info::next_account_info(account_info_iter)?;
        let b_vault_lp = solana_program::account_info::next_account_info(account_info_iter)?;
        let protocol_token_fee = solana_program::account_info::next_account_info(account_info_iter)?;
        let vault_program = solana_program::account_info::next_account_info(account_info_iter)?;
        let _token_program = solana_program::account_info::next_account_info(account_info_iter)?;
        if *meteora_v1_program.key != METEORA_V1_PROGRAM_ID || *vault_program.key != METEORA_V1_VAULT_PROGRAM_ID {
            return Err(Error::InvalidAccountPubkey.into());
        }
        if base_data.with_checks
            && (!pool.is_writable
                || !a_vault.is_writable
                || !b_vault.is_writable
                || !a_token_vault.is_writable
                || !b_token_vault.is_writable
                || !a_vault_lp_mint.is_writable
                || !b_vault_lp_mint.is_writable
                || !a_vault_lp.is_writable
                || !b_vault_lp.is_writable
                || !protocol_token_fee.is_writable)
        {
            return Err(Error::InvalidAccountConfigurationFlags.into());
        }
        // This is a slightly modified selective code from https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/dynamic-amm-quote/src/lib.rs#L58
        // that calculates the swap possibility.
        //
        // In ideal case all structures here should be deserealized from accounts in zero-copy context.
        // But in the source code the structures are serialized with 'borsh' and without zero-copy.
        let swap_calculation_result = {
            let clock_ = Clock::from_account_info(clock)?;
            let (trade_direction, trade_fee, protocol_fee) = match check_pool(pool, base_data, &clock_)? {
                Some(data) => data,
                None => return Ok(None),
            };
            let vault_a = Box::new(<Vault as borsh::de::BorshDeserialize>::deserialize(
                &mut &a_vault.data.borrow()[FIRST_BYTE_INDEX_AFTER_ANCHOR_DEFAULT_LENGTH_DISCRIMINATOR..],
            )?);
            let vault_b = Box::new(<Vault as borsh::de::BorshDeserialize>::deserialize(
                &mut &b_vault.data.borrow()[FIRST_BYTE_INDEX_AFTER_ANCHOR_DEFAULT_LENGTH_DISCRIMINATOR..],
            )?);
            let pool_vault_a_lp_token = Account::unpack_unchecked(&a_vault_lp.data.borrow())?.amount;
            let pool_vault_b_lp_token = Account::unpack_unchecked(&b_vault_lp.data.borrow())?.amount;
            let vault_a_lp_mint = Mint::unpack_unchecked(&a_vault_lp_mint.data.borrow())?.supply;
            let vault_b_lp_mint = Mint::unpack_unchecked(&b_vault_lp_mint.data.borrow())?.supply;
            let vault_a_token = Account::unpack_unchecked(&a_token_vault.data.borrow())?.amount;
            let vault_b_token = Account::unpack_unchecked(&b_token_vault.data.borrow())?.amount;
            // https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/rust-client/src/instructions/dynamic_amm/quote.rs#L79
            // https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-amm/src/instructions/swap.rs#L5
            //
            // MeteoraV1 swap instruction does not receive the 'pool_.stake' account to work with its data.
            let current_time: u64 = clock_.unix_timestamp.try_into().map_err(|_| ProgramError::ArithmeticOverflow)?;
            let token_a_amount = vault_a.get_amount_by_share(current_time, pool_vault_a_lp_token, vault_a_lp_mint).ok_or(ProgramError::ArithmeticOverflow)?;
            let token_b_amount = vault_b.get_amount_by_share(current_time, pool_vault_b_lp_token, vault_b_lp_mint).ok_or(ProgramError::ArithmeticOverflow)?;
            let (mut in_vault, out_vault, in_vault_lp, in_vault_lp_mint, out_vault_lp_mint, out_vault_token_account, in_token_total_amount, out_token_total_amount) =
                match trade_direction {
                    TradeDirection::AtoB => (vault_a, vault_b, pool_vault_a_lp_token, vault_a_lp_mint, vault_b_lp_mint, vault_b_token, token_a_amount, token_b_amount),
                    TradeDirection::BtoA => (vault_b, vault_a, pool_vault_b_lp_token, vault_b_lp_mint, vault_a_lp_mint, vault_a_token, token_b_amount, token_a_amount),
                };
            let trade_fee_: u64 = trade_fee.checked_sub(protocol_fee).ok_or(ProgramError::ArithmeticOverflow)?;
            let in_amount_after_protocol_fee =
                base_data.amount_in.checked_sub(protocol_fee).ok_or(ProgramError::ArithmeticOverflow)?;
            let before_in_token_total_amount = in_token_total_amount;
            let in_lp = in_vault.get_unmint_amount(current_time, in_amount_after_protocol_fee, in_vault_lp_mint).ok_or(ProgramError::ArithmeticOverflow)?;
            in_vault.total_amount = in_vault.total_amount.checked_add(in_amount_after_protocol_fee).ok_or(ProgramError::ArithmeticOverflow)?;
            let after_in_token_total_amount = in_vault
                .get_amount_by_share(
                    current_time,
                    in_lp.checked_add(in_vault_lp).ok_or(ProgramError::ArithmeticOverflow)?,
                    in_vault_lp_mint.checked_add(in_lp).ok_or(ProgramError::ArithmeticOverflow)?,
                )
                .ok_or(ProgramError::ArithmeticOverflow)?;
            let actual_in_amount = after_in_token_total_amount.checked_sub(before_in_token_total_amount).ok_or(ProgramError::ArithmeticOverflow)?;
            let actual_in_amount_after_fee = actual_in_amount.checked_sub(trade_fee_).ok_or(ProgramError::ArithmeticOverflow)?;
            let swap_curve = ConstantProduct;
            let SwapResult {
                destination_amount_swapped,
                ..
            } = swap_curve.swap(actual_in_amount_after_fee, in_token_total_amount, out_token_total_amount, trade_direction).ok_or(ProgramError::ArithmeticOverflow)?;
            let out_vault_lp = out_vault
                .get_unmint_amount(current_time, destination_amount_swapped.try_into().map_err(|_| ProgramError::ArithmeticOverflow)?, out_vault_lp_mint)
                .ok_or(ProgramError::ArithmeticOverflow)?;
            let amount_out_ = out_vault.get_amount_by_share(current_time, out_vault_lp, out_vault_lp_mint).ok_or(ProgramError::ArithmeticOverflow)?;
            if amount_out_ > out_vault_token_account {
                return Ok(None);
            }
            SwapCalculationResult {
                pool: *pool.key,
                amount_in_fee: trade_fee,
                amount_out: amount_out_,
            }
        };
        Ok(Some(swap_calculation_result))
    }
    fn do_swap(&'a self, base_data: &'a BaseData<'b, 'c>, first_account_index: usize) -> Result<(), ProgramError> {
        let account_info_iter = &mut base_data.accounts.iter().skip(first_account_index).take(self.get_swap_accounts_quantity());
        let meteora_v1_program = solana_program::account_info::next_account_info(account_info_iter)?;
        let _clock = solana_program::account_info::next_account_info(account_info_iter)?;
        let pool = solana_program::account_info::next_account_info(account_info_iter)?;
        let a_vault = solana_program::account_info::next_account_info(account_info_iter)?;
        let b_vault = solana_program::account_info::next_account_info(account_info_iter)?;
        let a_token_vault = solana_program::account_info::next_account_info(account_info_iter)?;
        let b_token_vault = solana_program::account_info::next_account_info(account_info_iter)?;
        let a_vault_lp_mint = solana_program::account_info::next_account_info(account_info_iter)?;
        let b_vault_lp_mint = solana_program::account_info::next_account_info(account_info_iter)?;
        let a_vault_lp = solana_program::account_info::next_account_info(account_info_iter)?;
        let b_vault_lp = solana_program::account_info::next_account_info(account_info_iter)?;
        let protocol_token_fee = solana_program::account_info::next_account_info(account_info_iter)?;
        let vault_program = solana_program::account_info::next_account_info(account_info_iter)?;
        let token_program = solana_program::account_info::next_account_info(account_info_iter)?;
        if base_data.is_from_quote_to_token {
            solana_program::program::invoke_signed(
                &Self::create_swap_instruction(
                    meteora_v1_program.key,
                    pool.key,
                    base_data.quote_token_account.key,
                    base_data.token_account.key,
                    a_vault.key,
                    b_vault.key,
                    a_token_vault.key,
                    b_token_vault.key,
                    a_vault_lp_mint.key,
                    b_vault_lp_mint.key,
                    a_vault_lp.key,
                    b_vault_lp.key,
                    protocol_token_fee.key,
                    base_data.self_authority.key,
                    vault_program.key,
                    token_program.key,
                    base_data.amount_in,
                    base_data.min_amount_out,
                ),
                vec![
                    pool.clone(),
                    base_data.quote_token_account.clone(),
                    base_data.token_account.clone(),
                    a_vault.clone(),
                    b_vault.clone(),
                    a_token_vault.clone(),
                    b_token_vault.clone(),
                    a_vault_lp_mint.clone(),
                    b_vault_lp_mint.clone(),
                    a_vault_lp.clone(),
                    b_vault_lp.clone(),
                    protocol_token_fee.clone(),
                    base_data.self_authority.clone(),
                    vault_program.clone(),
                    token_program.clone(),
                ]
                .as_slice(),
                [PdaResolver::self_authority_get_seeds(base_data.intermediary.key, [base_data.intermediary_.self_authority_pubkey_bump_seed].as_slice()).as_slice()].as_slice(),
            )
        } else {
            solana_program::program::invoke_signed(
                &Self::create_swap_instruction(
                    meteora_v1_program.key,
                    pool.key,
                    base_data.token_account.key,
                    base_data.quote_token_account.key,
                    a_vault.key,
                    b_vault.key,
                    a_token_vault.key,
                    b_token_vault.key,
                    a_vault_lp_mint.key,
                    b_vault_lp_mint.key,
                    a_vault_lp.key,
                    b_vault_lp.key,
                    protocol_token_fee.key,
                    base_data.self_authority.key,
                    vault_program.key,
                    token_program.key,
                    base_data.amount_in,
                    base_data.min_amount_out,
                ),
                vec![
                    pool.clone(),
                    base_data.token_account.clone(),
                    base_data.quote_token_account.clone(),
                    a_vault.clone(),
                    b_vault.clone(),
                    a_token_vault.clone(),
                    b_token_vault.clone(),
                    a_vault_lp_mint.clone(),
                    b_vault_lp_mint.clone(),
                    a_vault_lp.clone(),
                    b_vault_lp.clone(),
                    protocol_token_fee.clone(),
                    base_data.self_authority.clone(),
                    vault_program.clone(),
                    token_program.clone(),
                ]
                .as_slice(),
                [PdaResolver::self_authority_get_seeds(base_data.intermediary.key, [base_data.intermediary_.self_authority_pubkey_bump_seed].as_slice()).as_slice()].as_slice(),
            )
        }
    }
}
fn check_pool(pool: &AccountInfo, base_data: &BaseData, clock_: &Clock) -> Result<Option<(TradeDirection, u64, u64)>, ProgramError> {
    let in_token_mint = if base_data.is_from_quote_to_token {
        base_data.quote_mint
    } else {
        base_data.token_mint
    };
    let pool_ = Box::new(<Pool as borsh::de::BorshDeserialize>::deserialize(
        &mut &pool.data.borrow()[FIRST_BYTE_INDEX_AFTER_ANCHOR_DEFAULT_LENGTH_DISCRIMINATOR..],
    )?);
    if pool_.stake != Pubkey::default() {
        return Ok(None);
    }
    let activation_type = ActivationType::try_from(pool_.bootstrapping.activation_type).map_err(|_| ProgramError::InvalidArgument)?;
    let current_point = match activation_type {
        ActivationType::Slot => clock_.slot,
        ActivationType::Timestamp => clock_.unix_timestamp as u64,
    };
    if !pool_.enabled {
        return Ok(None);
    }
    if current_point < pool_.bootstrapping.activation_point {
        return Ok(None);
    }
    match pool_.curve_type {
        CurveType::ConstantProduct => {}
        // https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/dynamic-amm-quote/src/lib.rs#L91
        //
        // MeteoraV1 swap instruction does not receive the 'pool_.stake' account to work with its data.
        _ => return Ok(None),
    }
    if *in_token_mint != pool_.token_a_mint && *in_token_mint != pool_.token_b_mint {
        return Err(Error::InvalidTokenMint.into());
    }
    let trade_fee = pool_.fees.trading_fee(base_data.amount_in).ok_or(ProgramError::ArithmeticOverflow)?;
    let protocol_fee = pool_.fees.protocol_trading_fee(trade_fee).ok_or(ProgramError::ArithmeticOverflow)?;
    if *in_token_mint == pool_.token_a_mint {
        Ok(Some((TradeDirection::AtoB, trade_fee, protocol_fee)))
    } else {
        Ok(Some((TradeDirection::BtoA, trade_fee, protocol_fee)))
    }
}
