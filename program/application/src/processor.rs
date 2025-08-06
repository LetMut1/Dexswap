use {
    crate::{
        PROGRAM_ID,
        dex::{
            BaseData,
            Dex,
            meteora_v1::MeteoraV1,
            raydium_v4::RaydiumV4,
        },
        error::Error,
        instruction::Instruction,
        state::{
            Dex as Dex_,
            Intermediary,
            MUCH_USED_STATIC_ACCOUNTS,
            PdaResolver,
            QUANTITY_OF_MUCH_USED_DYNAMIC_ACCOUNTS,
        },
    },
    borsh::BorshDeserialize,
    solana_program::{
        account_info::AccountInfo,
        entrypoint::ProgramResult,
        msg,
        program_pack::Pack,
        pubkey::Pubkey,
        rent::Rent,
        sysvar::Sysvar,
    },
    spl_token::state::Account,
    std::{
        collections::BTreeSet,
        io::Write,
    },
};
pub struct Processor;
impl Processor {
    pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        match Instruction::try_from_slice(input)? {
            Instruction::Initialize {
                recent_slot,
                lamports_to_treasury,
                w_sol_token_account_pubkey_bump_seed,
                temporary_w_sol_token_account_pubkey_bump_seed,
                self_authority_pubkey_bump_seed,
            } => {
                Self::initialize(
                    accounts,
                    recent_slot,
                    lamports_to_treasury,
                    w_sol_token_account_pubkey_bump_seed,
                    temporary_w_sol_token_account_pubkey_bump_seed,
                    self_authority_pubkey_bump_seed,
                )
            }
            Instruction::DepositFunds {
                lamports_to_treasury,
            } => Self::deposit_funds(accounts, lamports_to_treasury),
            Instruction::WithdrawFunds {
                lamports_from_treasury,
            } => Self::withdraw_funds(accounts, lamports_from_treasury),
            Instruction::Swap {
                dexes,
                token_mint,
                quote_mint,
                amount_in,
                min_amount_out,
                token_account_pubkey_bump_seed,
                is_from_quote_to_token,
                with_checks,
            } => {
                Self::swap(
                    dexes,
                    accounts,
                    token_mint,
                    quote_mint,
                    amount_in,
                    min_amount_out,
                    token_account_pubkey_bump_seed,
                    is_from_quote_to_token,
                    with_checks,
                )
            }
        }
    }
    fn initialize(
        accounts: &[AccountInfo],
        recent_slot: u64,
        lamports_to_treasury: u64,
        w_sol_token_account_pubkey_bump_seed: u8,
        temporary_w_sol_token_account_pubkey_bump_seed: u8,
        self_authority_pubkey_bump_seed: u8,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let intermediary = solana_program::account_info::next_account_info(account_info_iter)?;
        let intermediary_manager = solana_program::account_info::next_account_info(account_info_iter)?;
        let intermediary_trader = solana_program::account_info::next_account_info(account_info_iter)?;
        let w_sol_token_account = solana_program::account_info::next_account_info(account_info_iter)?;
        let temporary_w_sol_token_account = solana_program::account_info::next_account_info(account_info_iter)?;
        let common_address_lookup_table = solana_program::account_info::next_account_info(account_info_iter)?;
        let self_authority = solana_program::account_info::next_account_info(account_info_iter)?;
        let w_sol_token_mint = solana_program::account_info::next_account_info(account_info_iter)?;
        let system_program = solana_program::account_info::next_account_info(account_info_iter)?;
        let rent = solana_program::account_info::next_account_info(account_info_iter)?;
        let token_program = solana_program::account_info::next_account_info(account_info_iter)?;
        let address_lookup_table_program = solana_program::account_info::next_account_info(account_info_iter)?;
        let (create_lookup_table_instruction, common_address_lookup_table_) =
            solana_program::address_lookup_table::instruction::create_lookup_table(*self_authority.key, *intermediary_manager.key, recent_slot);
        let mut account_differentiator = BTreeSet::<&Pubkey>::new();
        if !account_differentiator.insert(intermediary.key)
            || !account_differentiator.insert(intermediary_manager.key)
            || !account_differentiator.insert(intermediary_trader.key)
            || !account_differentiator.insert(w_sol_token_account.key)
            || !account_differentiator.insert(temporary_w_sol_token_account.key)
            || !account_differentiator.insert(common_address_lookup_table.key)
            || !account_differentiator.insert(self_authority.key)
            || !account_differentiator.insert(w_sol_token_mint.key)
            || !account_differentiator.insert(system_program.key)
            || !account_differentiator.insert(rent.key)
            || !account_differentiator.insert(token_program.key)
            || !account_differentiator.insert(address_lookup_table_program.key)
            || *w_sol_token_account.key != PdaResolver::token_account_create(intermediary.key, &spl_token::native_mint::ID, w_sol_token_account_pubkey_bump_seed)?
            || *temporary_w_sol_token_account.key != PdaResolver::temporary_w_sol_token_account_create(intermediary.key, temporary_w_sol_token_account_pubkey_bump_seed)?
            || *self_authority.key != PdaResolver::self_authority_create(intermediary.key, self_authority_pubkey_bump_seed)?
            || *common_address_lookup_table.key != common_address_lookup_table_
            || *w_sol_token_mint.key != spl_token::native_mint::ID
            || *system_program.key != solana_program::system_program::ID
            || *rent.key != solana_program::sysvar::rent::ID
            || *token_program.key != spl_token::ID
            || *address_lookup_table_program.key != solana_program::address_lookup_table::program::ID
        {
            return Err(Error::InvalidAccountPubkey.into());
        }
        let mut common_address_lookup_table_accounts = vec![
            *intermediary.key,
            *w_sol_token_account.key,
            *self_authority.key,
        ];
        if common_address_lookup_table_accounts.len() != QUANTITY_OF_MUCH_USED_DYNAMIC_ACCOUNTS as usize {
            return Err(Error::InvalidLogic.into());
        }
        account_differentiator = BTreeSet::<&Pubkey>::new();
        '_a: for account in MUCH_USED_STATIC_ACCOUNTS.iter() {
            if !account_differentiator.insert(account) {
                return Err(Error::InvalidLogic.into());
            }
            common_address_lookup_table_accounts.push(*account);
        }
        if !intermediary.is_writable
            || !intermediary.is_signer
            || !intermediary_manager.is_writable
            || !intermediary_manager.is_signer
            || !intermediary_trader.is_signer
            || !w_sol_token_account.is_writable
            || !common_address_lookup_table.is_writable
        {
            return Err(Error::InvalidAccountConfigurationFlags.into());
        }
        if intermediary_manager.lamports() == 0 || intermediary_trader.lamports() == 0 {
            return Err(Error::InvalidAccountLamports.into());
        }
        let intermediary_ = Intermediary::new(
            *intermediary_manager.key,
            *intermediary_trader.key,
            *w_sol_token_account.key,
            *temporary_w_sol_token_account.key,
            *common_address_lookup_table.key,
            *self_authority.key,
            w_sol_token_account_pubkey_bump_seed,
            temporary_w_sol_token_account_pubkey_bump_seed,
            self_authority_pubkey_bump_seed,
        );
        let intermediary_object_length = std::mem::size_of::<Intermediary>();
        let rent_ = Rent::from_account_info(rent)?;
        let intermediary_rent_exemption_balance = rent_.minimum_balance(intermediary_object_length);
        let token_account_rent_exemption_balance = rent_.minimum_balance(<Account as Pack>::LEN);
        solana_program::program::invoke(
            &solana_program::system_instruction::create_account(
                intermediary_manager.key,
                intermediary.key,
                intermediary_rent_exemption_balance,
                intermediary_object_length as u64,
                &PROGRAM_ID,
            ),
            vec![
                intermediary_manager.clone(),
                intermediary.clone(),
            ]
            .as_slice(),
        )?;
        (&mut intermediary.data.borrow_mut()[..]).write_all(bytemuck::bytes_of(&intermediary_))?;
        solana_program::program::invoke_signed(
            &solana_program::system_instruction::create_account(
                intermediary_manager.key,
                w_sol_token_account.key,
                token_account_rent_exemption_balance + lamports_to_treasury,
                <Account as Pack>::LEN as u64,
                token_program.key,
            ),
            vec![
                intermediary_manager.clone(),
                w_sol_token_account.clone(),
            ]
            .as_slice(),
            [PdaResolver::token_account_get_seeds(intermediary.key, &spl_token::native_mint::ID, [w_sol_token_account_pubkey_bump_seed].as_slice()).as_slice()].as_slice(),
        )?;
        solana_program::program::invoke(
            &spl_token::instruction::initialize_account(token_program.key, w_sol_token_account.key, w_sol_token_mint.key, self_authority.key)?,
            vec![
                w_sol_token_account.clone(),
                w_sol_token_mint.clone(),
                self_authority.clone(),
                rent.clone(),
            ]
            .as_slice(),
        )?;
        solana_program::program::invoke(
            &create_lookup_table_instruction,
            vec![
                common_address_lookup_table.clone(),
                self_authority.clone(),
                intermediary_manager.clone(),
                system_program.clone(),
            ]
            .as_slice(),
        )?;
        solana_program::program::invoke_signed(
            &solana_program::address_lookup_table::instruction::extend_lookup_table(
                *common_address_lookup_table.key,
                *self_authority.key,
                Some(*intermediary_manager.key),
                common_address_lookup_table_accounts,
            ),
            vec![
                common_address_lookup_table.clone(),
                self_authority.clone(),
                intermediary_manager.clone(),
                system_program.clone(),
            ]
            .as_slice(),
            [PdaResolver::self_authority_get_seeds(intermediary.key, [self_authority_pubkey_bump_seed].as_slice()).as_slice()].as_slice(),
        )?;
        Ok(())
    }
    fn deposit_funds(accounts: &[AccountInfo], lamports_to_treasury: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let intermediary = solana_program::account_info::next_account_info(account_info_iter)?;
        let intermediary_manager = solana_program::account_info::next_account_info(account_info_iter)?;
        let w_sol_token_account = solana_program::account_info::next_account_info(account_info_iter)?;
        let system_program = solana_program::account_info::next_account_info(account_info_iter)?;
        let token_program = solana_program::account_info::next_account_info(account_info_iter)?;
        if *system_program.key != solana_program::system_program::ID || *token_program.key != spl_token::ID {
            return Err(Error::InvalidAccountPubkey.into());
        }
        if !intermediary_manager.is_writable || !intermediary_manager.is_signer || !w_sol_token_account.is_writable {
            return Err(Error::InvalidAccountConfigurationFlags.into());
        }
        if intermediary_manager.lamports() < lamports_to_treasury {
            return Err(Error::InvalidAccountLamports.into());
        }
        let intermediary_data = &intermediary.data.borrow();
        let intermediary_ = bytemuck::try_from_bytes::<Intermediary>(intermediary_data).map_err(|_| Error::InvalidLogic)?;
        if !intermediary_.is_initialized() {
            return Err(Error::IntermediaryIsNotInitialized.into());
        }
        if *intermediary_manager.key != intermediary_.manager {
            return Err(Error::IntermediaryInvalidManager.into());
        }
        if *w_sol_token_account.key != intermediary_.w_sol_token_account
            || *w_sol_token_account.key != PdaResolver::token_account_create(intermediary.key, &spl_token::native_mint::ID, intermediary_.w_sol_token_account_pubkey_bump_seed)?
        {
            return Err(Error::IntermediaryInvalidWSolTokenAccount.into());
        }
        solana_program::program::invoke(
            &solana_program::system_instruction::transfer(intermediary_manager.key, w_sol_token_account.key, lamports_to_treasury),
            vec![
                intermediary_manager.clone(),
                w_sol_token_account.clone(),
            ]
            .as_slice(),
        )?;
        solana_program::program::invoke(
            &spl_token::instruction::sync_native(token_program.key, w_sol_token_account.key)?,
            vec![
                w_sol_token_account.clone(),
            ]
            .as_slice(),
        )?;
        Ok(())
    }
    fn withdraw_funds(accounts: &[AccountInfo], lamports_from_treasury: u64) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let intermediary = solana_program::account_info::next_account_info(account_info_iter)?;
        let intermediary_manager = solana_program::account_info::next_account_info(account_info_iter)?;
        let w_sol_token_account = solana_program::account_info::next_account_info(account_info_iter)?;
        let temporary_w_sol_token_account = solana_program::account_info::next_account_info(account_info_iter)?;
        let self_authority = solana_program::account_info::next_account_info(account_info_iter)?;
        let w_sol_token_mint = solana_program::account_info::next_account_info(account_info_iter)?;
        let system_program = solana_program::account_info::next_account_info(account_info_iter)?;
        let rent = solana_program::account_info::next_account_info(account_info_iter)?;
        let token_program = solana_program::account_info::next_account_info(account_info_iter)?;
        if *w_sol_token_mint.key != spl_token::native_mint::ID
            || *system_program.key != solana_program::system_program::ID
            || *rent.key != solana_program::sysvar::rent::ID
            || *token_program.key != spl_token::ID
        {
            return Err(Error::InvalidAccountPubkey.into());
        }
        if !intermediary_manager.is_writable || !intermediary_manager.is_signer || !w_sol_token_account.is_writable || !temporary_w_sol_token_account.is_writable {
            return Err(Error::InvalidAccountConfigurationFlags.into());
        }
        let intermediary_data = &intermediary.data.borrow();
        let intermediary_ = bytemuck::try_from_bytes::<Intermediary>(intermediary_data).map_err(|_| Error::InvalidLogic)?;
        if !intermediary_.is_initialized() {
            return Err(Error::IntermediaryIsNotInitialized.into());
        }
        if *intermediary_manager.key != intermediary_.manager {
            return Err(Error::IntermediaryInvalidManager.into());
        }
        if *w_sol_token_account.key != intermediary_.w_sol_token_account
            || *w_sol_token_account.key != PdaResolver::token_account_create(intermediary.key, &spl_token::native_mint::ID, intermediary_.w_sol_token_account_pubkey_bump_seed)?
        {
            return Err(Error::IntermediaryInvalidWSolTokenAccount.into());
        }
        if *temporary_w_sol_token_account.key != intermediary_.temporary_w_sol_token_account
            || *temporary_w_sol_token_account.key
                != PdaResolver::temporary_w_sol_token_account_create(intermediary.key, intermediary_.temporary_w_sol_token_account_pubkey_bump_seed)?
        {
            return Err(Error::IntermediaryInvalidTemporaryWSolTokenAccount.into());
        }
        if lamports_from_treasury > Account::unpack_unchecked(&w_sol_token_account.data.borrow())?.amount {
            return Err(Error::TokenAccountInsufficientAmount.into());
        }
        if *self_authority.key != intermediary_.self_authority
            || *self_authority.key != PdaResolver::self_authority_create(intermediary.key, intermediary_.self_authority_pubkey_bump_seed)?
        {
            return Err(Error::IntermediaryInvalidAuthority.into());
        }
        let rent_ = Rent::from_account_info(rent)?;
        let token_account_rent_exemption_balance = rent_.minimum_balance(<Account as Pack>::LEN);
        if intermediary_manager.lamports() < token_account_rent_exemption_balance {
            return Err(Error::InvalidAccountLamports.into());
        }
        solana_program::program::invoke_signed(
            &solana_program::system_instruction::create_account(
                intermediary_manager.key,
                temporary_w_sol_token_account.key,
                token_account_rent_exemption_balance,
                <Account as Pack>::LEN as u64,
                token_program.key,
            ),
            vec![
                intermediary_manager.clone(),
                temporary_w_sol_token_account.clone(),
            ]
            .as_slice(),
            [PdaResolver::temporary_w_sol_token_account_get_seeds(intermediary.key, [intermediary_.temporary_w_sol_token_account_pubkey_bump_seed].as_slice()).as_slice()]
                .as_slice(),
        )?;
        solana_program::program::invoke(
            &spl_token::instruction::initialize_account(token_program.key, temporary_w_sol_token_account.key, w_sol_token_mint.key, intermediary_manager.key)?,
            vec![
                temporary_w_sol_token_account.clone(),
                w_sol_token_mint.clone(),
                intermediary_manager.clone(),
                rent.clone(),
            ]
            .as_slice(),
        )?;
        solana_program::program::invoke_signed(
            &spl_token::instruction::transfer(
                token_program.key,
                w_sol_token_account.key,
                temporary_w_sol_token_account.key,
                self_authority.key,
                [].as_slice(),
                lamports_from_treasury,
            )?,
            vec![
                w_sol_token_account.clone(),
                temporary_w_sol_token_account.clone(),
                self_authority.clone(),
            ]
            .as_slice(),
            [PdaResolver::self_authority_get_seeds(intermediary.key, [intermediary_.self_authority_pubkey_bump_seed].as_slice()).as_slice()].as_slice(),
        )?;
        solana_program::program::invoke(
            &spl_token::instruction::close_account(token_program.key, temporary_w_sol_token_account.key, intermediary_manager.key, intermediary_manager.key, [].as_slice())?,
            vec![
                temporary_w_sol_token_account.clone(),
                intermediary_manager.clone(),
                intermediary_manager.clone(),
            ]
            .as_slice(),
        )?;
        Ok(())
    }
    fn swap(
        dexes: Vec<Dex_>,
        accounts: &[AccountInfo],
        token_mint: Pubkey,
        quote_mint: Pubkey,
        amount_in: u64,
        min_amount_out: u64,
        token_account_pubkey_bump_seed: u8,
        is_from_quote_to_token: bool,
        with_checks: bool,
    ) -> ProgramResult {
        // Only direction from WSol to AnyMint is valid.
        if !is_from_quote_to_token || quote_mint != spl_token::native_mint::ID {
            return Err(Error::NotImplemented.into());
        }
        if token_mint == quote_mint {
            return Err(Error::EqualMints.into());
        }
        if amount_in == 0 {
            return Err(Error::ZeroAmountIn.into());
        }
        if dexes.is_empty() {
            return Err(Error::ZeroDexesPresented.into());
        }
        let account_info_iter = &mut accounts.iter();
        let intermediary = solana_program::account_info::next_account_info(account_info_iter)?;
        let intermediary_trader = solana_program::account_info::next_account_info(account_info_iter)?;
        let quote_token_account = solana_program::account_info::next_account_info(account_info_iter)?;
        let self_authority = solana_program::account_info::next_account_info(account_info_iter)?;
        let token_account = solana_program::account_info::next_account_info(account_info_iter)?;
        let quote_token_mint = solana_program::account_info::next_account_info(account_info_iter)?;
        let token_mint_ = solana_program::account_info::next_account_info(account_info_iter)?;
        let system_program = solana_program::account_info::next_account_info(account_info_iter)?;
        let rent = solana_program::account_info::next_account_info(account_info_iter)?;
        let token_program = solana_program::account_info::next_account_info(account_info_iter)?;
        if token_mint != *token_mint_.key {
            return Err(Error::InvalidTokenMint.into());
        }
        if with_checks {
            if *token_account.key != PdaResolver::token_account_create(intermediary.key, token_mint_.key, token_account_pubkey_bump_seed)?
                || quote_token_mint.key == token_mint_.key
                || *quote_token_mint.key != spl_token::native_mint::ID
                || *system_program.key != solana_program::system_program::ID
                || *rent.key != solana_program::sysvar::rent::ID
                || *token_program.key != spl_token::ID
            {
                return Err(Error::InvalidAccountPubkey.into());
            }
            if !intermediary_trader.is_signer || !intermediary_trader.is_writable || !quote_token_account.is_writable || !token_account.is_writable {
                return Err(Error::InvalidAccountConfigurationFlags.into());
            }
        }
        let rent_ = Rent::from_account_info(rent)?;
        let token_account_rent_exemption_balance = rent_.minimum_balance(<Account as Pack>::LEN);
        if intermediary_trader.lamports() < token_account_rent_exemption_balance {
            return Err(Error::InvalidAccountLamports.into());
        }
        let intermediary_data = &intermediary.data.borrow();
        let intermediary_ = bytemuck::try_from_bytes::<Intermediary>(intermediary_data).map_err(|_| Error::InvalidLogic)?;
        if !intermediary_.is_initialized() {
            return Err(Error::IntermediaryIsNotInitialized.into());
        }
        if *intermediary_trader.key != intermediary_.trader {
            return Err(Error::IntermediaryInvalidTrader.into());
        }
        if *quote_token_account.key != intermediary_.w_sol_token_account
            || *quote_token_account.key != PdaResolver::token_account_create(intermediary.key, &spl_token::native_mint::ID, intermediary_.w_sol_token_account_pubkey_bump_seed)?
        {
            return Err(Error::IntermediaryInvalidWSolTokenAccount.into());
        }
        if *self_authority.key != intermediary_.self_authority
            || *self_authority.key != PdaResolver::self_authority_create(intermediary.key, intermediary_.self_authority_pubkey_bump_seed)?
        {
            return Err(Error::IntermediaryInvalidAuthority.into());
        }
        let initial_quote_token_amount = Account::unpack_unchecked(&quote_token_account.data.borrow())?.amount;
        if amount_in > initial_quote_token_amount {
            return Err(Error::TokenAccountInsufficientAmount.into());
        }
        let mut initial_token_amount = 0;
        if token_account.data_is_empty() {
            solana_program::program::invoke_signed(
                &solana_program::system_instruction::create_account(
                    intermediary_trader.key,
                    token_account.key,
                    token_account_rent_exemption_balance,
                    <Account as Pack>::LEN as u64,
                    token_program.key,
                ),
                vec![
                    intermediary_trader.clone(),
                    token_account.clone(),
                ]
                .as_slice(),
                [PdaResolver::token_account_get_seeds(intermediary.key, token_mint_.key, [token_account_pubkey_bump_seed].as_slice()).as_slice()].as_slice(),
            )?;
            solana_program::program::invoke(
                &spl_token::instruction::initialize_account(
                    token_program.key,
                    token_account.key,
                    token_mint_.key,
                    // Owner - intermediary.self_authority. That is, all manipulations with reducing the token amount
                    // on the account are carried out through a this contract.
                    self_authority.key,
                )?,
                vec![
                    token_account.clone(),
                    token_mint_.clone(),
                    self_authority.clone(),
                    rent.clone(),
                ]
                .as_slice(),
            )?;
        } else {
            initial_token_amount = Account::unpack_unchecked(&token_account.data.borrow())?.amount
        }
        let base_data = BaseData {
            accounts,
            intermediary,
            quote_token_account,
            token_account,
            self_authority,
            intermediary_,
            token_mint: &token_mint,
            quote_mint: &quote_mint,
            amount_in,
            min_amount_out,
            is_from_quote_to_token,
            with_checks,
        };
        let mut dexes_ = Vec::<(Dex_, &dyn Dex)>::with_capacity(dexes.len());
        let mut dexes_btree_set = BTreeSet::<Dex_>::new();
        '_a: for dex in dexes {
            if !dexes_btree_set.insert(dex) {
                return Err(Error::RepeatableDex.into());
            }
            let dex_ = match dex {
                Dex_::MeteoraV1 => &MeteoraV1 as &dyn Dex,
                Dex_::RaydiumV4 => &RaydiumV4,
            };
            dexes_.push((dex, dex_));
        }
        const INTERMEDIARY_RESERVED_ACCOUNTS_QUANTUTY: usize = 10;
        let mut first_account_index = INTERMEDIARY_RESERVED_ACCOUNTS_QUANTUTY;
        let mut previous_dex_swap_accounts_quantity: usize = 0;
        let mut dex_with_swap_calculation_result = None;
        'a: for (dex_, dex) in dexes_.into_iter() {
            first_account_index += previous_dex_swap_accounts_quantity;
            previous_dex_swap_accounts_quantity += dex.get_swap_accounts_quantity();
            // Here returns Error, because we believe that the discrepancy between the data on the accounts
            // and the method signature parameters is a logical error, and we cannot simply move on to the next Dex.
            let swap_calculation_result = match dex.do_swap_calculation(&base_data, first_account_index) {
                Ok(swap_calculation_result_) => swap_calculation_result_,
                Err(program_error) => {
                    msg!("0Fail. Invalid CPI accounts for Dex {},", dex_.to_str());
                    return Err(program_error);
                }
            };
            let swap_calculation_result_ = match swap_calculation_result {
                Some(swap_calculation_result__) => swap_calculation_result__,
                None => continue 'a,
            };
            if swap_calculation_result_.amount_out >= min_amount_out {
                dex_with_swap_calculation_result = Some((dex_, swap_calculation_result_));
                if let Err(program_error) = dex.do_swap(&base_data, first_account_index) {
                    msg!("1Fail. Invalid CPI accounts for Dex {},", dex_.to_str());
                    return Err(program_error);
                }
                break 'a;
            } else {
                continue 'a;
            }
        }
        match dex_with_swap_calculation_result {
            Some(dex_with_swap_calculation_result_) => {
                solana_program::program::invoke(
                    &spl_token::instruction::sync_native(token_program.key, quote_token_account.key)?,
                    vec![
                        quote_token_account.clone(),
                    ]
                    .as_slice(),
                )?;
                let new_token_amount = Account::unpack_unchecked(&token_account.data.borrow())?.amount;
                if Account::unpack_unchecked(&quote_token_account.data.borrow())?.amount < (initial_quote_token_amount - amount_in)
                    || (new_token_amount as u128) < (initial_token_amount as u128 + min_amount_out as u128)
                {
                    msg!(
                        "2Fail. Invalid calculation logic. Dex: {}, pool: {}, in_mint: {}, out_mint: {}, amount_in : {}, amount_in_fee: {}, amount_out: {}, min_amount_out: {}.",
                        dex_with_swap_calculation_result_.0.to_str(),
                        &dex_with_swap_calculation_result_.1.pool,
                        &quote_mint,
                        &token_mint,
                        amount_in,
                        dex_with_swap_calculation_result_.1.amount_in_fee,
                        dex_with_swap_calculation_result_.1.amount_out,
                        min_amount_out,
                    );
                    return Err(Error::TokenAccountInvalidAmount.into());
                }
                msg!(
                    "0Success. Dex: {}, pool: {}, in_mint: {}, out_mint: {}, amount_in : {}, amount_in_fee: {}, amount_out: {}, min_amount_out: {}.",
                    dex_with_swap_calculation_result_.0.to_str(),
                    &dex_with_swap_calculation_result_.1.pool,
                    &quote_mint,
                    &token_mint,
                    amount_in,
                    dex_with_swap_calculation_result_.1.amount_in_fee,
                    new_token_amount - initial_token_amount,
                    min_amount_out,
                )
            }
            None => {
                msg!(
                    "3Fail. No matching dex found. In_mint: {}, out_mint: {}, amount_in : {}, min_amount_out: {}.",
                    &quote_mint,
                    &token_mint,
                    amount_in,
                    min_amount_out,

                );
                return Err(Error::InvalidSwapConditions.into());
            }
        }
        Ok(())
    }
}
