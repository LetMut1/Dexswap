use {
    super::CheckedCeilDiv,
    solana_program::pubkey::Pubkey,
};
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-amm/src/state.rs#L61
#[derive(borsh::BorshSchema, borsh::BorshDeserialize)]
pub struct Pool {
    /// LP token mint of the pool
    pub lp_mint: Pubkey, //32
    /// Token A mint of the pool. Eg: USDT
    pub token_a_mint: Pubkey, //32
    /// Token B mint of the pool. Eg: USDC
    pub token_b_mint: Pubkey, //32
    /// Vault account for token A. Token A of the pool will be deposit / withdraw from this vault account.
    pub a_vault: Pubkey, //32
    /// Vault account for token B. Token B of the pool will be deposit / withdraw from this vault account.
    pub b_vault: Pubkey, //32
    /// LP token account of vault A. Used to receive/burn the vault LP upon deposit/withdraw from the vault.
    pub a_vault_lp: Pubkey, //32
    /// LP token account of vault B. Used to receive/burn the vault LP upon deposit/withdraw from the vault.
    pub b_vault_lp: Pubkey, //32
    /// "A" vault lp bump. Used to create signer seeds.
    pub a_vault_lp_bump: u8, //1
    /// Flag to determine whether the pool is enabled, or disabled.
    pub enabled: bool, //1
    /// Protocol fee token account for token A. Used to receive trading fee.
    pub protocol_token_a_fee: Pubkey, //32
    /// Protocol fee token account for token B. Used to receive trading fee.
    pub protocol_token_b_fee: Pubkey, //32
    /// Fee last updated timestamp
    pub fee_last_updated_at: u64,
    // Padding leftover from deprecated admin pubkey. Beware of tombstone when reusing it.
    pub _padding0: [u8; 24],
    /// Store the fee charges setting.
    pub fees: PoolFees, //48
    /// Pool type
    pub pool_type: PoolType,
    /// Stake pubkey of SPL stake pool
    pub stake: Pubkey,
    /// Total locked lp token
    pub total_locked_lp: u64,
    /// Bootstrapping config
    pub bootstrapping: Bootstrapping,
    pub partner_info: PartnerInfo,
    /// Padding for future pool field
    pub padding: Padding,
    /// The type of the swap curve supported by the pool.
    // Leaving curve_type as last field give us the flexibility to add specific curve information / new curve type
    pub curve_type: CurveType, //9
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-amm/src/state.rs#L172
#[derive(borsh::BorshSchema, borsh::BorshDeserialize)]
pub struct PoolFees {
    /// Trade fees are extra token amounts that are held inside the token
    /// accounts during a trade, making the value of liquidity tokens rise.
    /// Trade fee numerator
    pub trade_fee_numerator: u64,
    /// Trade fee denominator
    pub trade_fee_denominator: u64,

    /// Owner trading fees are extra token amounts that are held inside the token
    /// accounts during a trade, with the equivalent in pool tokens minted to
    /// the owner of the program.
    /// Owner trade fee numerator
    pub protocol_trade_fee_numerator: u64,
    /// Owner trade fee denominator
    pub protocol_trade_fee_denominator: u64,
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-amm/src/state.rs#L208
impl PoolFees {
    /// Calculate the host trading fee in trading tokens
    pub fn host_trading_fee(&self, trading_tokens: u128) -> Option<u128> {
        // Floor division
        trading_tokens.checked_mul(HOST_TRADE_FEE_NUMERATOR.into())?.checked_div(FEE_DENOMINATOR.into())
    }
    /// Calculate the trading fee in trading tokens
    pub fn trading_fee(&self, trading_tokens: u64) -> Option<u64> {
        calculate_fee(trading_tokens, self.trade_fee_numerator, self.trade_fee_denominator)
    }
    /// Calculate the protocol trading fee in trading tokens
    pub fn protocol_trading_fee(&self, trading_tokens: u64) -> Option<u64> {
        calculate_fee(trading_tokens, self.protocol_trade_fee_numerator, self.protocol_trade_fee_denominator)
    }
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-amm/src/constants.rs#L34
pub const HOST_TRADE_FEE_NUMERATOR: u64 = 20000;
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-amm/src/constants.rs#L37
pub const FEE_DENOMINATOR: u64 = 100000;
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-amm/src/state.rs#L190
pub fn calculate_fee(token_amount: u64, fee_numerator: u64, fee_denominator: u64) -> Option<u64> {
    if fee_numerator == 0 || token_amount == 0 {
        Some(0)
    } else {
        let fee = (token_amount as u128).checked_mul(fee_numerator as u128)?.checked_div(fee_denominator as u128)?;
        if fee == 0 {
            Some(1) // minimum fee of one token
        } else {
            Some(fee as u64)
        }
    }
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-amm/src/state.rs#L46
#[derive(borsh::BorshSchema, borsh::BorshDeserialize)]
pub enum PoolType {
    /// Permissioned
    Permissioned,
    /// Permissionless
    Permissionless,
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-amm/src/state.rs#L135
#[derive(borsh::BorshSchema, borsh::BorshDeserialize)]
pub struct Bootstrapping {
    /// Activation point, can be slot or timestamp
    pub activation_point: u64,
    /// Whitelisted vault to be able to buy pool before open slot
    pub whitelisted_vault: Pubkey,
    pub pool_creator: Pubkey,
    /// Activation type, 0 means by slot, 1 means by timestamp
    pub activation_type: u8,
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-amm/src/state.rs#L107
#[derive(borsh::BorshSchema, borsh::BorshDeserialize)]
pub struct PartnerInfo {
    pub fee_numerator: u64,
    pub partner_authority: Pubkey,
    pub pending_fee_a: u64,
    pub pending_fee_b: u64,
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-amm/src/state.rs#L27
#[derive(borsh::BorshSchema, borsh::BorshDeserialize)]
pub struct Padding {
    /// Padding 0
    pub padding_0: [u8; 6], // 6
    /// Padding 1
    pub padding_1: [u64; 21], // 168
    /// Padding 2
    pub padding_2: [u64; 21], // 168
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-amm/src/state.rs#L238
#[derive(borsh::BorshSchema, borsh::BorshDeserialize)]
pub enum CurveType {
    /// Uniswap-style constant product curve, invariant = token_a_amount * token_b_amount
    ConstantProduct,
    /// Stable, like uniswap, but with wide zone of 1:1 instead of one point
    Stable {
        /// Amplification coefficient
        amp: u64,
        /// Multiplier for the pool token. Used to normalized token with different decimal into the same precision.
        token_multiplier: TokenMultiplier,
        /// Depeg pool information. Contains functions to allow token amount to be repeg using stake / interest bearing token virtual price
        depeg: Depeg,
        /// The last amp updated timestamp. Used to prevent update_curve_info called infinitely many times within a short period
        last_amp_updated_timestamp: u64,
    },
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-amm/src/state.rs#L256
#[derive(borsh::BorshSchema, borsh::BorshDeserialize)]
pub struct TokenMultiplier {
    /// Multiplier for token A of the pool.
    pub token_a_multiplier: u64, // 8
    /// Multiplier for token B of the pool.
    pub token_b_multiplier: u64, // 8
    /// Record the highest token decimal in the pool. For example, Token A is 6 decimal, token B is 9 decimal. This will save value of 9.
    pub precision_factor: u8, // 1
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-amm/src/state.rs#L286
#[derive(borsh::BorshSchema, borsh::BorshDeserialize)]
pub struct Depeg {
    /// The virtual price of staking / interest bearing token
    pub base_virtual_price: u64,
    /// The virtual price of staking / interest bearing token
    pub base_cache_updated: u64,
    /// Type of the depeg pool
    pub depeg_type: DepegType,
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-amm/src/state.rs#L304
#[derive(borsh::BorshSchema, borsh::BorshDeserialize)]
pub enum DepegType {
    /// Indicate that it is not a depeg pool
    None,
    /// A depeg pool belongs to marinade finance
    Marinade,
    /// A depeg pool belongs to solido
    Lido,
    /// A depeg pool belongs to SPL stake pool program
    SplStake,
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-amm/src/state.rs#L295
impl DepegType {
    /// Check whether the pool is a depeg pool or not
    pub fn is_none(&self) -> bool {
        matches!(self, DepegType::None)
    }
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-vault/src/state.rs#L16
#[derive(borsh::BorshSchema, borsh::BorshDeserialize)]
pub struct Vault {
    /// The flag, if admin set enable = false, then the user can only withdraw and cannot deposit in the vault.
    pub enabled: u8,
    /// Vault nonce, to create vault seeds
    pub bumps: VaultBumps,
    /// The total liquidity of the vault, including remaining tokens in token_vault and the liquidity in all strategies.
    pub total_amount: u64,
    /// Token account, hold liquidity in vault reserve
    pub token_vault: Pubkey,
    /// Hold lp token of vault, each time rebalance crank is called, vault calculate performance fee and mint corresponding lp token amount to fee_vault. fee_vault is owned by treasury address
    pub fee_vault: Pubkey,
    /// Token mint that vault supports
    pub token_mint: Pubkey,
    /// Lp mint of vault
    pub lp_mint: Pubkey,
    /// The list of strategy addresses that vault supports, vault can support up to MAX_STRATEGY strategies at the same time.
    pub strategies: [Pubkey; MAX_STRATEGY],
    /// The base address to create vault seeds
    pub base: Pubkey,
    /// Admin of vault
    pub admin: Pubkey,
    /// Person who can send the crank. Operator can only send liquidity to strategies that admin defined, and claim reward to account of treasury address
    pub operator: Pubkey,
    /// Stores information for locked profit.
    pub locked_profit_tracker: LockedProfitTracker,
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-vault/src/state.rs#L43
impl Vault {
    /// Get amount by share
    pub fn get_amount_by_share(&self, current_time: u64, share: u64, total_supply: u64) -> Option<u64> {
        let total_amount = self.get_unlocked_amount(current_time)?;
        u64::try_from(u128::from(share).checked_mul(u128::from(total_amount))?.checked_div(u128::from(total_supply))?).ok()
    }
    /// Get unlocked amount of vault
    pub fn get_unlocked_amount(&self, current_time: u64) -> Option<u64> {
        self.total_amount.checked_sub(self.locked_profit_tracker.calculate_locked_profit(current_time)?)
    }
    /// Get unmint amount by token amount
    pub fn get_unmint_amount(&self, current_time: u64, out_token: u64, total_supply: u64) -> Option<u64> {
        let total_amount = self.get_unlocked_amount(current_time)?;
        u64::try_from(u128::from(out_token).checked_mul(u128::from(total_supply))?.checked_div(u128::from(total_amount))?).ok()
    }
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-vault/src/state.rs#L7
pub const MAX_STRATEGY: usize = 30;
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-vault/src/state.rs#L143
#[derive(borsh::BorshSchema, borsh::BorshDeserialize)]
pub struct VaultBumps {
    /// vault_bump
    pub vault_bump: u8,
    /// token_vault_bump
    pub token_vault_bump: u8,
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-vault/src/state.rs#L86
#[derive(borsh::BorshSchema, borsh::BorshDeserialize)]
pub struct LockedProfitTracker {
    /// The total locked profit from the last report
    pub last_updated_locked_profit: u64,
    /// The last timestamp (in seconds) rebalancing
    pub last_report: u64,
    /// Rate per second of degradation
    pub locked_profit_degradation: u64,
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-vault/src/state.rs#L95
impl LockedProfitTracker {
    /// Calculate locked profit, based from Yearn `https://github.com/yearn/yearn-vaults/blob/main/contracts/Vault.vy#L825`
    pub fn calculate_locked_profit(&self, current_time: u64) -> Option<u64> {
        let duration = u128::from(current_time.checked_sub(self.last_report)?);
        let locked_profit_degradation = u128::from(self.locked_profit_degradation);
        let locked_fund_ratio = duration.checked_mul(locked_profit_degradation)?;
        if locked_fund_ratio > LOCKED_PROFIT_DEGRADATION_DENOMINATOR {
            return Some(0);
        }
        let locked_profit = u128::from(self.last_updated_locked_profit);
        let locked_profit = (locked_profit.checked_mul(LOCKED_PROFIT_DEGRADATION_DENOMINATOR - locked_fund_ratio)?).checked_div(LOCKED_PROFIT_DEGRADATION_DENOMINATOR)?;
        let locked_profit = u64::try_from(locked_profit).ok()?;
        Some(locked_profit)
    }
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-vault/src/state.rs#L11
pub const LOCKED_PROFIT_DEGRADATION_DENOMINATOR: u128 = 1_000_000_000_000;
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/programs/dynamic-amm/src/state.rs#L117
#[repr(u8)]
/// Type of the activation
pub enum ActivationType {
    Slot,
    Timestamp,
}
impl TryFrom<u8> for ActivationType {
    type Error = String;
    fn try_from(s: u8) -> std::result::Result<ActivationType, String> {
        match s {
            0 => Ok(ActivationType::Slot),
            1 => Ok(ActivationType::Timestamp),
            _ => Err("Invalid value".to_string()),
        }
    }
}
pub enum TradeDirection {
    /// Input token A, output token B
    AtoB,
    /// Input token B, output token A
    BtoA,
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/dynamic-amm-quote/src/math/mod.rs#L12
#[derive(Debug, PartialEq)]
pub struct SwapResult {
    /// New amount of source token
    pub new_swap_source_amount: u128,
    /// New amount of destination token
    pub new_swap_destination_amount: u128,
    /// Amount of source token swapped (includes fees)
    pub source_amount_swapped: u128,
    /// Amount of destination token swapped
    pub destination_amount_swapped: u128,
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/dynamic-amm-quote/src/math/mod.rs#L23
pub trait SwapCurve {
    fn swap(&self, source_amount: u64, swap_source_amount: u64, swap_destination_amount: u64, trade_direction: TradeDirection) -> Option<SwapResult>;
}
pub struct ConstantProduct;
impl ConstantProduct {
    // https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/dynamic-amm-quote/src/math/constant_product.rs#L2
    fn swap_(source_amount: u128, swap_source_amount: u128, swap_destination_amount: u128) -> Option<SwapWithoutFeesResult> {
        let invariant = swap_source_amount.checked_mul(swap_destination_amount)?;
        let new_swap_source_amount = swap_source_amount.checked_add(source_amount)?;
        let (new_swap_destination_amount, new_swap_source_amount) = invariant.checked_ceil_div(new_swap_source_amount)?;
        let source_amount_swapped = new_swap_source_amount.checked_sub(swap_source_amount)?;
        let destination_amount_swapped = map_zero_to_none(swap_destination_amount.checked_sub(new_swap_destination_amount)?)?;
        Some(SwapWithoutFeesResult {
            source_amount_swapped,
            destination_amount_swapped,
        })
    }
}
impl SwapCurve for ConstantProduct {
    fn swap(&self, source_amount: u64, swap_source_amount: u64, swap_destination_amount: u64, _trade_direction: TradeDirection) -> Option<SwapResult> {
        let source_amount: u128 = source_amount.into();
        let swap_source_amount: u128 = swap_source_amount.into();
        let swap_destination_amount: u128 = swap_destination_amount.into();
        let SwapWithoutFeesResult {
            source_amount_swapped,
            destination_amount_swapped,
        } = Self::swap_(source_amount, swap_source_amount, swap_destination_amount)?;
        Some(SwapResult {
            new_swap_source_amount: swap_source_amount.checked_add(source_amount_swapped)?,
            new_swap_destination_amount: swap_destination_amount.checked_sub(destination_amount_swapped)?,
            source_amount_swapped,
            destination_amount_swapped,
        })
    }
}
pub fn map_zero_to_none(x: u128) -> Option<u128> {
    if x == 0 {
        None
    } else {
        Some(x)
    }
}
// https://github.com/MeteoraAg/damm-v1-sdk/blob/b21e2efb3680c17a68149ed2e22465aeef9b3784/dynamic-amm-quote/src/math/constant_product.rs#L2
pub struct SwapWithoutFeesResult {
    /// Amount of source token swapped
    pub source_amount_swapped: u128,
    /// Amount of destination token swapped
    pub destination_amount_swapped: u128,
}
impl CheckedCeilDiv for u128 {
    fn checked_ceil_div(&self, mut rhs: Self) -> Option<(Self, Self)> {
        let mut quotient = self.checked_div(rhs)?;
        // Avoid dividing a small number by a big one and returning 1, and instead
        // fail.
        if quotient == 0 {
            return None;
        }
        // Ceiling the destination amount if there's any remainder, which will
        // almost always be the case.
        let remainder = self.checked_rem(rhs)?;
        if remainder > 0 {
            quotient = quotient.checked_add(1)?;
            // calculate the minimum amount needed to get the dividend amount to
            // avoid truncating too much
            rhs = self.checked_div(quotient)?;
            let remainder = self.checked_rem(quotient)?;
            if remainder > 0 {
                rhs = rhs.checked_add(1)?;
            }
        }
        Some((quotient, rhs))
    }
}
