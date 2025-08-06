use {
    crate::utility::Loader,
    intermediary::{
        METEORA_V1_PROGRAM_ID,
        METEORA_V1_VAULT_PROGRAM_ID,
        PROGRAM_ID,
        extern_source::meteora_v1::{
            Pool,
            Vault,
        },
        instruction::{
            Dex_,
            Instruction,
        },
        state::{
            Intermediary,
            MUCH_USED_STATIC_ACCOUNTS,
            PdaResolver,
            QUANTITY_OF_MUCH_USED_DYNAMIC_ACCOUNTS,
        },
    },
    solana_program::{
        address_lookup_table::state::LOOKUP_TABLE_META_SIZE,
        program_pack::Pack,
    },
    solana_rpc_client::rpc_client::RpcClient,
    solana_sdk::{
        address_lookup_table::state::AddressLookupTable,
        commitment_config::{
            CommitmentConfig,
            CommitmentLevel,
        },
        message::{
            AddressLookupTableAccount,
            VersionedMessage,
            legacy::Message,
            v0::Message as Message_,
        },
        pubkey::Pubkey,
        signer::{
            Signer,
            keypair::Keypair,
        },
        transaction::{
            Transaction,
            VersionedTransaction,
        },
    },
    spl_token::state::Account,
    std::{
        error::Error,
        str::FromStr,
    },
};
pub struct CommandProcessor;
impl CommandProcessor {
    const ERROR_INTERMEDIARY_IS_NOT_INITIALIZED: &'static str = "Intermediary is not initialized.";
    const ERROR_INTERMEDIARY_INVALID_MANAGER: &'static str = "Intermediary invalid manager.";
    const ERROR_INTERMEDIARY_INVALID_TRADER: &'static str = "Intermediary invalid trader.";
    const ERROR_INVALID_ACCOUNT_LAMPORTS: &'static str = "Invalid account lamports.";
    const ERROR_INVALID_ACCOUNT_PUBKEY: &'static str = "Invalid account pubkey.";
    pub fn initialize(
        rpc_client: &RpcClient,
        intermediary_manager_keypair_file_path: &str,
        intermediary_trader_keypair_file_path: &str,
        lamports_to_treasury: u64,
    ) -> Result<(), Box<dyn Error + 'static>> {
        let intermediary_manager_keypair = Loader::load_keypair_from_file(intermediary_manager_keypair_file_path)?;
        let intermediary_manager = intermediary_manager_keypair.pubkey();
        let intermediary_trader_keypair = Loader::load_keypair_from_file(intermediary_trader_keypair_file_path)?;
        let intermediary_trader = intermediary_trader_keypair.pubkey();
        if intermediary_manager == intermediary_trader {
            return Err(Self::ERROR_INVALID_ACCOUNT_PUBKEY.into());
        }
        let intermediary_manager_account = rpc_client.get_account(&intermediary_manager)?;
        let intermediary_trader_account = rpc_client.get_account(&intermediary_trader)?;
        let accounts_in_address_lookup_table_quantity = QUANTITY_OF_MUCH_USED_DYNAMIC_ACCOUNTS as usize + MUCH_USED_STATIC_ACCOUNTS.len();
        if accounts_in_address_lookup_table_quantity > ADDRESS_LOOKUP_TABLE_INDEXES_MAXIMUM_QUANTITY_FOR_TRANSACTION as usize {
            return Err("The safe implementation for multiple transactions is required.".into());
        }
        let address_lookop_table_size = LOOKUP_TABLE_META_SIZE + accounts_in_address_lookup_table_quantity * Pubkey::default().to_bytes().len();
        let intermediary_balance_for_rent_exemption = rpc_client.get_minimum_balance_for_rent_exemption(std::mem::size_of::<Intermediary>())?;
        let w_sol_token_account_rent_exemption_balance = rpc_client.get_minimum_balance_for_rent_exemption(<Account as Pack>::LEN)?;
        let address_lookop_table_balance_for_rent_exemption = rpc_client.get_minimum_balance_for_rent_exemption(address_lookop_table_size)?;
        if (intermediary_manager_account.lamports as u128)
            < (intermediary_balance_for_rent_exemption as u128
                + w_sol_token_account_rent_exemption_balance as u128
                + lamports_to_treasury as u128
                + address_lookop_table_balance_for_rent_exemption as u128)
            || intermediary_trader_account.lamports == 0
        {
            return Err(Self::ERROR_INVALID_ACCOUNT_LAMPORTS.into());
        }
        let intermediary_keypair = Keypair::new();
        let intermediary = intermediary_keypair.pubkey();
        println!("intermediary: {}", &intermediary);
        let (w_sol_token_account, w_sol_token_account_pubkey_bump_seed) = PdaResolver::token_account_find(&intermediary, &spl_token::native_mint::ID);
        println!("w_sol_token_account: {}", &w_sol_token_account);
        let (temporary_w_sol_token_account, temporary_w_sol_token_account_pubkey_bump_seed) = PdaResolver::temporary_w_sol_token_account_find(&intermediary);
        println!("temporary_w_sol_token_account: {}", &temporary_w_sol_token_account);
        let (self_account_authority, self_authority_pubkey_bump_seed) = PdaResolver::self_authority_find(&intermediary);
        println!("self_account_authority: {}", &self_account_authority);
        let recent_slot = rpc_client.get_slot_with_commitment(CommitmentConfig {
            commitment: CommitmentLevel::Finalized,
        })?;
        let (common_address_lookup_table, _) = solana_program::address_lookup_table::instruction::derive_lookup_table_address(&self_account_authority, recent_slot);
        println!("common_address_lookup_table: {}", &common_address_lookup_table);
        // 32 - in ALT documentation.
        const ADDRESS_LOOKUP_TABLE_INDEXES_MAXIMUM_QUANTITY_FOR_TRANSACTION: u8 = 32;
        let instructions = vec![
            Instruction::initialize(
                &PROGRAM_ID,
                &intermediary,
                &intermediary_manager,
                &intermediary_trader,
                &w_sol_token_account,
                &temporary_w_sol_token_account,
                &common_address_lookup_table,
                &self_account_authority,
                &spl_token::native_mint::ID,
                &solana_program::system_program::ID,
                &solana_program::sysvar::rent::ID,
                &spl_token::ID,
                &solana_program::address_lookup_table::program::ID,
                recent_slot,
                lamports_to_treasury,
                w_sol_token_account_pubkey_bump_seed,
                temporary_w_sol_token_account_pubkey_bump_seed,
                self_authority_pubkey_bump_seed,
            )?,
        ];
        let signers = vec![&intermediary_keypair, &intermediary_manager_keypair, &intermediary_trader_keypair];
        let recent_blockhash = rpc_client.get_latest_blockhash()?;
        let message = Message::new_with_blockhash(instructions.as_slice(), Some(&intermediary_manager), &recent_blockhash);
        let transaction = Transaction::new(signers.as_slice(), message, recent_blockhash);
        let signature = rpc_client.send_transaction(&transaction)?;
        println!("Signature: {}", &signature);
        Ok(())
    }
    pub fn deposit_funds(
        rpc_client: &RpcClient,
        intermediary_pubkey: &str,
        intermediary_manager_keypair_file_path: &str,
        lamports_to_treasury: u64,
    ) -> Result<(), Box<dyn Error + 'static>> {
        let intermediary_manager_keypair = Loader::load_keypair_from_file(intermediary_manager_keypair_file_path)?;
        let intermediary_manager = intermediary_manager_keypair.pubkey();
        let intermediary = Pubkey::from_str(intermediary_pubkey)?;
        let intermediary_manager_account = rpc_client.get_account(&intermediary_manager)?;
        if intermediary_manager_account.lamports < lamports_to_treasury {
            return Err(Self::ERROR_INVALID_ACCOUNT_LAMPORTS.into());
        }
        let intermediary_account = rpc_client.get_account(&intermediary)?;
        let intermediary_data = intermediary_account.data.as_slice();
        let intermediary_ = bytemuck::from_bytes::<Intermediary>(intermediary_data);
        if !intermediary_.is_initialized() {
            return Err(Self::ERROR_INTERMEDIARY_IS_NOT_INITIALIZED.into());
        }
        if intermediary_manager != intermediary_.manager {
            return Err(Self::ERROR_INTERMEDIARY_INVALID_MANAGER.into());
        }
        let instructions = vec![
            Instruction::deposit_funds(
                &PROGRAM_ID,
                &intermediary,
                &intermediary_manager,
                &intermediary_.w_sol_token_account,
                &solana_program::system_program::ID,
                &spl_token::ID,
                lamports_to_treasury,
            )?,
        ];
        let signers = vec![&intermediary_manager_keypair];
        let recent_blockhash = rpc_client.get_latest_blockhash()?;
        let message = Message::new_with_blockhash(instructions.as_slice(), Some(&intermediary_manager), &recent_blockhash);
        let transaction = Transaction::new(signers.as_slice(), message, recent_blockhash);
        let signature = rpc_client.send_transaction(&transaction)?;
        println!("Signature: {}", &signature);
        Ok(())
    }
    pub fn withdraw_funds(
        rpc_client: &RpcClient,
        intermediary_pubkey: &str,
        intermediary_manager_keypair_file_path: &str,
        lamports_from_treasury: u64,
    ) -> Result<(), Box<dyn Error + 'static>> {
        let intermediary_manager_keypair = Loader::load_keypair_from_file(intermediary_manager_keypair_file_path)?;
        let intermediary_manager = intermediary_manager_keypair.pubkey();
        let intermediary = Pubkey::from_str(intermediary_pubkey)?;
        let intermediary_manager_account = rpc_client.get_account(&intermediary_manager)?;
        let temporary_w_sol_token_account_rent_exemption_balance = rpc_client.get_minimum_balance_for_rent_exemption(<Account as Pack>::LEN)?;
        if intermediary_manager_account.lamports < temporary_w_sol_token_account_rent_exemption_balance {
            return Err(Self::ERROR_INVALID_ACCOUNT_LAMPORTS.into());
        }
        let intermediary_account = rpc_client.get_account(&intermediary)?;
        let intermediary_data = intermediary_account.data.as_slice();
        let intermediary_ = bytemuck::from_bytes::<Intermediary>(intermediary_data);
        if !intermediary_.is_initialized() {
            return Err(Self::ERROR_INTERMEDIARY_IS_NOT_INITIALIZED.into());
        }
        if intermediary_manager != intermediary_.manager {
            return Err(Self::ERROR_INTERMEDIARY_INVALID_MANAGER.into());
        }
        let w_sol_token_account = Account::unpack_unchecked(rpc_client.get_account(&intermediary_.w_sol_token_account)?.data.as_slice())?;
        if w_sol_token_account.amount < lamports_from_treasury {
            return Err(format!(
                    "The maximum number of lamports from treasury is {}",
                    w_sol_token_account.amount,
                )
            .into());
        }
        let instructions = vec![
            Instruction::withdraw_funds(
                &PROGRAM_ID,
                &intermediary,
                &intermediary_manager,
                &intermediary_.w_sol_token_account,
                &intermediary_.temporary_w_sol_token_account,
                &intermediary_.self_authority,
                &spl_token::native_mint::ID,
                &solana_program::system_program::ID,
                &solana_program::sysvar::rent::ID,
                &spl_token::ID,
                lamports_from_treasury,
            )?,
        ];
        let signers = vec![&intermediary_manager_keypair];
        let recent_blockhash = rpc_client.get_latest_blockhash()?;
        let message = Message::new_with_blockhash(instructions.as_slice(), Some(&intermediary_manager), &recent_blockhash);
        let transaction = Transaction::new(signers.as_slice(), message, recent_blockhash);
        let signature = rpc_client.send_transaction(&transaction)?;
        println!("Signature: {}", &signature);
        Ok(())
    }
    pub fn swap(
        rpc_client: &RpcClient,
        intermediary_pubkey: &str,
        intermediary_trader_keypair_file_path: &str,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<(), Box<dyn Error + 'static>> {
        let intermediary_trader_keypair = Loader::load_keypair_from_file(intermediary_trader_keypair_file_path)?;
        let intermediary_trader = intermediary_trader_keypair.pubkey();
        let intermediary = Pubkey::from_str(intermediary_pubkey)?;
        let intermediary_account = rpc_client.get_account(&intermediary)?;
        let intermediary_data = intermediary_account.data.as_slice();
        let intermediary_ = bytemuck::from_bytes::<Intermediary>(intermediary_data);
        if !intermediary_.is_initialized() {
            return Err(Self::ERROR_INTERMEDIARY_IS_NOT_INITIALIZED.into());
        }
        if intermediary_trader != intermediary_.trader {
            return Err(Self::ERROR_INTERMEDIARY_INVALID_TRADER.into());
        }
        let common_address_lookup_table_account = rpc_client.get_account(&intermediary_.common_address_lookup_table)?;
        let common_address_lookup_table = AddressLookupTable::deserialize(common_address_lookup_table_account.data.as_slice())?;
        if common_address_lookup_table.addresses.len() != QUANTITY_OF_MUCH_USED_DYNAMIC_ACCOUNTS as usize + MUCH_USED_STATIC_ACCOUNTS.len() {
            return Err("Invalid common_address_lookup_table account state.".into());
        }
        let pool_pubkey = Pubkey::from_str_const("4RnP2XvmeN21nCz8tPspGEiSTzBJcsMn1eeY5mzm8N1d");
        let pool_account = rpc_client.get_account(&pool_pubkey)?;
        const FIRST_BYTE_INDEX_AFTER_ANCHOR_DEFAULT_LENGTH_DISCRIMINATOR: usize = 8;
        let pool = <Pool as borsh::de::BorshDeserialize>::deserialize(&mut &(pool_account.data.as_slice()[FIRST_BYTE_INDEX_AFTER_ANCHOR_DEFAULT_LENGTH_DISCRIMINATOR..])).unwrap();
        let a_vault_account = rpc_client.get_account(&pool.a_vault)?;
        let a_vault =
            <Vault as borsh::de::BorshDeserialize>::deserialize(&mut &(a_vault_account.data.as_slice()[FIRST_BYTE_INDEX_AFTER_ANCHOR_DEFAULT_LENGTH_DISCRIMINATOR..])).unwrap();
        let b_vault_account = rpc_client.get_account(&pool.b_vault)?;
        let b_vault =
            <Vault as borsh::de::BorshDeserialize>::deserialize(&mut &(b_vault_account.data.as_slice()[FIRST_BYTE_INDEX_AFTER_ANCHOR_DEFAULT_LENGTH_DISCRIMINATOR..])).unwrap();
        let quote_mint = spl_token::native_mint::id();
        let (token_mint, protocol_token_fee) = if a_vault.token_mint == quote_mint {
            (b_vault.token_mint, pool.protocol_token_a_fee)
        } else {
            (a_vault.token_mint, pool.protocol_token_b_fee)
        };
        let (token_account, token_account_pubkey_bump_seed) = PdaResolver::token_account_find(&intermediary, &token_mint);
        let instructions = vec![
            Instruction::swap(
                &PROGRAM_ID,
                &intermediary,
                &intermediary_trader,
                &intermediary_.w_sol_token_account,
                &intermediary_.self_authority,
                &token_account,
                &quote_mint,
                &token_mint,
                &solana_program::system_program::ID,
                &solana_program::sysvar::rent::ID,
                &spl_token::ID,
                vec![
                    Dex_::MeteoraV1 {
                        meteora_v1_program: &METEORA_V1_PROGRAM_ID,
                        clock:  &solana_program::sysvar::clock::ID,
                        pool: &pool_pubkey,
                        a_vault: &pool.a_vault,
                        b_vault: &pool.b_vault,
                        a_token_vault: &a_vault.token_vault,
                        b_token_vault: &b_vault.token_vault,
                        a_vault_lp_mint: &a_vault.lp_mint,
                        b_vault_lp_mint: &b_vault.lp_mint,
                        a_vault_lp: &pool.a_vault_lp,
                        b_vault_lp: &pool.b_vault_lp,
                        protocol_token_fee: &protocol_token_fee,
                        vault_program: &METEORA_V1_VAULT_PROGRAM_ID,
                        token_program: &spl_token::ID,
                    },
                ],
                amount_in,
                min_amount_out,
                token_account_pubkey_bump_seed,
                true,
                true,

            )?,
        ];
        let common_address_lookup_table_account_ = AddressLookupTableAccount {
            key: intermediary_.common_address_lookup_table,
            addresses: common_address_lookup_table.addresses.to_vec(),
        };
        let signers = vec![&intermediary_trader_keypair];
        let recent_blockhash = rpc_client.get_latest_blockhash()?;
        let message = Message_::try_compile(&intermediary_trader, instructions.as_slice(), [common_address_lookup_table_account_].as_slice(), recent_blockhash)?;
        let versioned_transaction = VersionedTransaction::try_new(VersionedMessage::V0(message), signers.as_slice())?;
        let signature = rpc_client.send_transaction(&versioned_transaction)?;
        println!("Signature: {}", &signature);
        Ok(())
    }
}
