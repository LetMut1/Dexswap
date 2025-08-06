#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::manual_range_patterns)]
#![allow(clippy::result_unit_err)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::reversed_empty_ranges)]
#![allow(clippy::explicit_auto_deref)]
use {
    super::CheckedCeilDiv,
    crate::error::Error,
    arrayref::{
        array_mut_ref,
        array_ref,
        array_refs,
        mut_array_refs,
    },
    bytemuck::{
        Pod,
        Zeroable,
        cast,
        cast_slice_mut,
        from_bytes,
        from_bytes_mut,
        try_cast_slice_mut,
        try_from_bytes_mut,
    },
    enumflags2::BitFlags,
    num_enum::{
        IntoPrimitive,
        TryFromPrimitive,
    },
    safe_transmute::{
        self,
        trivial::TriviallyTransmutable,
    },
    serde::{
        Deserialize,
        Serialize,
    },
    solana_program::{
        account_info::AccountInfo,
        program_error::ProgramError,
        program_pack::{
            IsInitialized,
            Pack,
            Sealed,
        },
        pubkey::Pubkey,
    },
    spl_token::state::Account,
    std::{
        cell::{
            Ref,
            RefMut,
        },
        convert::identity,
        mem::size_of,
        num::NonZeroU64,
        ops::{
            Deref,
            DerefMut,
        },
    },
    uint::construct_uint,
};
// https://github.com/raydium-io/raydium-amm/blob/2748852a7981c2b6909e07e10b1325669fbb9195/program/src/math.rs#L294
// This function copied from source without any changes in purpose to disable logs in 'calc_exact_vault_in_serum()'
pub fn calc_total_without_take_pnl(
    pc_amount: u64,
    coin_amount: u64,
    open_orders: &OpenOrders,
    amm: &AmmInfo,
    market_state: &MarketState,
    event_q_account: &AccountInfo,
    amm_open_account: &AccountInfo,
) -> Result<(u64, u64), Error> {
    let (pc_total_in_serum, coin_total_in_serum) = calc_exact_vault_in_serum(open_orders, market_state, event_q_account, amm_open_account)?;
    let total_pc_without_take_pnl = pc_amount
        .checked_add(pc_total_in_serum)
        .ok_or(Error::CheckedAddOverflow)?
        .checked_sub(amm.state_data.need_take_pnl_pc)
        .ok_or(Error::CheckedSubOverflow)?;
    let total_coin_without_take_pnl = coin_amount
        .checked_add(coin_total_in_serum)
        .ok_or(Error::CheckedAddOverflow)?
        .checked_sub(amm.state_data.need_take_pnl_coin)
        .ok_or(Error::CheckedSubOverflow)?;
    Ok((total_pc_without_take_pnl, total_coin_without_take_pnl))
}
// https://github.com/raydium-io/raydium-amm/blob/2748852a7981c2b6909e07e10b1325669fbb9195/program/src/math.rs#L244
// This function copied from source without any changes in purpose to disable logs.
pub fn calc_exact_vault_in_serum(open_orders: &OpenOrders, market_state: &MarketState, event_q_account: &AccountInfo, amm_open_account: &AccountInfo) -> Result<(u64, u64), Error> {
    let event_q = market_state.load_event_queue_mut(event_q_account).unwrap();
    let mut native_pc_total = open_orders.native_pc_total;
    let mut native_coin_total = open_orders.native_coin_total;
    for event in event_q.iter() {
        if identity(event.owner) != (*amm_open_account.key).to_aligned_bytes() {
            continue;
        }
        match event.as_view().unwrap() {
            EventView::Fill {
                side,
                maker,
                native_qty_paid,
                native_qty_received,
                native_fee_or_rebate: _,
                fee_tier: _,
                order_id: _,
                owner: _,
                owner_slot: _,
                client_order_id: _,
            } => {
                match side {
                    Side::Bid if maker => {
                        native_pc_total -= native_qty_paid;
                        native_coin_total += native_qty_received;
                    }
                    Side::Ask if maker => {
                        native_coin_total -= native_qty_paid;
                        native_pc_total += native_qty_received;
                    }
                    _ => (),
                };
            }
            _ => {
                continue;
            }
        }
    }
    Ok((native_pc_total, native_coin_total))
}
pub fn load_serum_market_order<'a>(
    market_acc: &AccountInfo<'a>,
    open_orders_acc: &AccountInfo<'a>,
    authority_acc: &AccountInfo<'a>,
    amm: &AmmInfo,
    // Allow for the market flag to be set to AccountFlag::Disabled
    allow_disabled: bool,
) -> Result<(Box<MarketState>, Box<OpenOrders>), ProgramError> {
    let market_state = Market::load_checked(market_acc, &amm.market_program, allow_disabled)?;
    let open_orders = OpenOrders::load_checked(open_orders_acc, Some(market_acc), Some(authority_acc), &amm.market_program)?;
    if identity(open_orders.market) != market_acc.key.to_aligned_bytes() {
        return Err(Error::InvalidMarket.into());
    }
    if identity(open_orders.owner) != authority_acc.key.to_aligned_bytes() {
        return Err(Error::InvalidOwner.into());
    }
    if *open_orders_acc.key != amm.open_orders {
        return Err(Error::InvalidOpenOrders.into());
    }
    return Ok((Box::new(*market_state.deref()), Box::new(*open_orders.deref())));
}
pub enum Market<'a> {
    V1(RefMut<'a, MarketState>),
    V2(RefMut<'a, MarketStateV2>),
    V1Ref(Ref<'a, MarketState>),
    V2Ref(Ref<'a, MarketStateV2>),
}
impl<'a> Deref for Market<'a> {
    type Target = MarketState;
    fn deref(&self) -> &Self::Target {
        match self {
            Market::V1(v1) => v1.deref(),
            Market::V2(v2) => v2.deref(),
            Market::V1Ref(v1_ref) => v1_ref.deref(),
            Market::V2Ref(v2_ref) => v2_ref.deref(),
        }
    }
}
impl<'a> DerefMut for Market<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Market::V1(v1) => v1.deref_mut(),
            Market::V2(v2) => v2.deref_mut(),
            _ => unreachable!(),
        }
    }
}
impl<'a> Market<'a> {
    #[inline]
    pub fn load_checked(
        market_account: &'a AccountInfo,
        program_id: &Pubkey,
        // Allow for the market flag to be set to AccountFlag::Disabled
        allow_disabled: bool,
    ) -> Result<Self, Error> {
        let flags = Market::account_flags(&market_account.try_borrow_data().map_err(|_| Error::InvalidLogic)?)?;
        if flags.intersects(AccountFlag::Permissioned) {
            Ok(Market::V2Ref(MarketStateV2::load_checked(market_account, program_id, allow_disabled)?))
        } else {
            Ok(Market::V1Ref(MarketState::load_checked(market_account, program_id, allow_disabled)?))
        }
    }
    pub fn account_flags(account_data: &[u8]) -> Result<BitFlags<AccountFlag>, Error> {
        let start = ACCOUNT_HEAD_PADDING.len();
        let end = start + size_of::<AccountFlag>();
        if account_data.len() < end {
            return Err(Error::InvalidLogic);
        }
        let mut flag_bytes = [0u8; 8];
        flag_bytes.copy_from_slice(&account_data[start..end]);
        BitFlags::from_bits(u64::from_le_bytes(flag_bytes)).map_err(|_| Error::InvalidLogic).map(Into::into)
    }
}
impl MarketState {
    #[inline]
    pub fn load_checked<'a>(market_account: &'a AccountInfo, program_id: &Pubkey, allow_disabled: bool) -> Result<Ref<'a, Self>, Error> {
        if market_account.owner != program_id {
            return Err(Error::InvalidLogic);
        }
        let account_data = market_account.try_borrow_data().map_err(|_| Error::InvalidLogic)?;
        if account_data.len() < 12 {
            return Err(Error::InvalidLogic);
        }
        let head = array_ref![account_data, 0, 5];
        let tail = array_ref![account_data, account_data.len() - 7, 7];
        if head != ACCOUNT_HEAD_PADDING {
            return Err(Error::InvalidLogic);
        }
        if tail != ACCOUNT_TAIL_PADDING {
            return Err(Error::InvalidLogic);
        }
        let state: Ref<'a, Self> = Ref::map(account_data, |account_data| bytemuck::from_bytes(&account_data[5..account_data.len() - 7]));
        state.check_flags(allow_disabled)?;
        Ok(state)
    }
    #[inline]
    pub fn check_flags(&self, allow_disabled: bool) -> Result<(), Error> {
        let flags = BitFlags::from_bits(self.account_flags).map_err(|_| Error::InvalidLogic)?;
        let required_flags = AccountFlag::Initialized | AccountFlag::Market;
        if allow_disabled {
            let disabled_flags = required_flags | AccountFlag::Disabled;
            if flags != required_flags && flags != disabled_flags {
                return Err(Error::InvalidLogic);
            }
        } else {
            if flags != required_flags {
                return Err(Error::InvalidLogic);
            }
        }
        Ok(())
    }
}
#[derive(Copy, Clone)]
#[cfg_attr(
    target_endian = "little",
    derive(Debug)
)]
#[repr(packed)]
pub struct MarketStateV2 {
    pub inner: MarketState,
    pub open_orders_authority: Pubkey,
    pub prune_authority: Pubkey,
    pub consume_events_authority: Pubkey,
    // Unused bytes for future upgrades.
    padding: [u8; 992],
}
pub const ACCOUNT_HEAD_PADDING: &[u8; 5] = b"serum";
pub const ACCOUNT_TAIL_PADDING: &[u8; 7] = b"padding";
impl MarketStateV2 {
    #[inline]
    pub fn load_checked<'a>(market_account: &'a AccountInfo, program_id: &Pubkey, allow_disabled: bool) -> Result<Ref<'a, Self>, Error> {
        if market_account.owner != program_id {
            return Err(Error::InvalidLogic);
        }
        let account_data = market_account.try_borrow_data().map_err(|_| Error::InvalidLogic)?;
        if account_data.len() < 12 {
            return Err(Error::InvalidLogic);
        }
        let head = array_ref![account_data, 0, 5];
        let tail = array_ref![account_data, account_data.len() - 7, 7];
        if head != ACCOUNT_HEAD_PADDING {
            return Err(Error::InvalidLogic);
        }
        if tail != ACCOUNT_TAIL_PADDING {
            return Err(Error::InvalidLogic);
        }
        let state: Ref<'a, Self> = Ref::map(account_data, |account_data| bytemuck::from_bytes(&account_data[5..account_data.len() - 7]));
        state.check_flags(allow_disabled)?;
        Ok(state)
    }
    #[inline]
    pub fn check_flags(&self, allow_disabled: bool) -> Result<(), Error> {
        let flags = BitFlags::from_bits(self.account_flags).map_err(|_| Error::InvalidLogic)?;
        let required_flags = AccountFlag::Initialized | AccountFlag::Market | AccountFlag::Permissioned;
        let required_crank_flags = required_flags | AccountFlag::CrankAuthorityRequired;
        if allow_disabled {
            let disabled_flags = required_flags | AccountFlag::Disabled;
            let disabled_crank_flags = required_crank_flags | AccountFlag::Disabled;
            if flags != required_flags && flags != required_crank_flags && flags != disabled_flags && flags != disabled_crank_flags {
                return Err(Error::InvalidLogic);
            }
        } else {
            if flags != required_flags && flags != required_crank_flags {
                return Err(Error::InvalidLogic);
            }
        }
        Ok(())
    }
}
impl Deref for MarketStateV2 {
    type Target = MarketState;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for MarketStateV2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
unsafe impl Zeroable for MarketStateV2 {}
unsafe impl Pod for MarketStateV2 {}
unsafe impl TriviallyTransmutable for MarketStateV2 {}
// https://github.com/raydium-io/raydium-amm/blob/2748852a7981c2b6909e07e10b1325669fbb9195/program/src/state.rs#L639
#[repr(
    C,
    packed
)]
#[derive(Clone, Copy, Default, PartialEq)]
pub struct AmmInfo {
    /// Initialized status.
    pub status: u64,
    /// Nonce used in program address.
    /// The program address is created deterministically with the nonce,
    /// amm program id, and amm account pubkey.  This program address has
    /// authority over the amm's token coin account, token pc account, and pool
    /// token mint.
    pub nonce: u64,
    /// max order count
    pub order_num: u64,
    /// within this range, 5 => 5% range
    pub depth: u64,
    /// coin decimal
    pub coin_decimals: u64,
    /// pc decimal
    pub pc_decimals: u64,
    /// amm machine state
    pub state: u64,
    /// amm reset_flag
    pub reset_flag: u64,
    /// min size 1->0.000001
    pub min_size: u64,
    /// vol_max_cut_ratio numerator, sys_decimal_value as denominator
    pub vol_max_cut_ratio: u64,
    /// amount wave numerator, sys_decimal_value as denominator
    pub amount_wave: u64,
    /// coinLotSize 1 -> 0.000001
    pub coin_lot_size: u64,
    /// pcLotSize 1 -> 0.000001
    pub pc_lot_size: u64,
    /// min_cur_price: (2 * amm.order_num * amm.pc_lot_size) * max_price_multiplier
    pub min_price_multiplier: u64,
    /// max_cur_price: (2 * amm.order_num * amm.pc_lot_size) * max_price_multiplier
    pub max_price_multiplier: u64,
    /// system decimal value, used to normalize the value of coin and pc amount
    pub sys_decimal_value: u64,
    /// All fee information
    pub fees: Fees,
    /// Statistical data
    pub state_data: StateData,
    /// Coin vault
    pub coin_vault: Pubkey,
    /// Pc vault
    pub pc_vault: Pubkey,
    /// Coin vault mint
    pub coin_vault_mint: Pubkey,
    /// Pc vault mint
    pub pc_vault_mint: Pubkey,
    /// lp mint
    pub lp_mint: Pubkey,
    /// open_orders key
    pub open_orders: Pubkey,
    /// market key
    pub market: Pubkey,
    /// market program key
    pub market_program: Pubkey,
    /// target_orders key
    pub target_orders: Pubkey,
    /// padding
    pub padding1: [u64; 8],
    /// amm owner key
    pub amm_owner: Pubkey,
    /// pool lp amount
    pub lp_amount: u64,
    /// client order id
    pub client_order_id: u64,
    /// recent epoch
    pub recent_epoch: u64,
    /// padding
    pub padding2: u64,
}
impl AmmInfo {
    /// Helper function to get the more efficient packed size of the struct
    /// load_mut_checked
    #[inline]
    pub fn load_mut_checked<'a>(account: &'a AccountInfo, program_id: &Pubkey) -> Result<RefMut<'a, Self>, ProgramError> {
        if account.owner != program_id {
            return Err(Error::InvalidAmmAccountOwner.into());
        }
        if account.data_len() != size_of::<Self>() {
            return Err(Error::ExpectedAccount.into());
        }
        let data = Self::load_mut(account)?;
        if data.status == AmmStatus::Uninitialized as u64 {
            return Err(Error::InvalidStatus.into());
        }
        Ok(data)
    }
    /// load_checked
    #[inline]
    pub fn load_checked<'a>(account: &'a AccountInfo, program_id: &Pubkey) -> Result<Ref<'a, Self>, ProgramError> {
        if account.owner != program_id {
            return Err(Error::InvalidAmmAccountOwner.into());
        }
        if account.data_len() != size_of::<Self>() {
            return Err(Error::ExpectedAccount.into());
        }
        let data = Self::load(account)?;
        if data.status == AmmStatus::Uninitialized as u64 {
            return Err(Error::InvalidStatus.into());
        }
        Ok(data)
    }
}
#[repr(
    C,
    packed
)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StateData {
    /// delay to take pnl coin
    pub need_take_pnl_coin: u64,
    /// delay to take pnl pc
    pub need_take_pnl_pc: u64,
    /// total pnl pc
    pub total_pnl_pc: u64,
    /// total pnl coin
    pub total_pnl_coin: u64,
    /// ido pool open time
    pub pool_open_time: u64,
    /// padding for future updates
    pub padding: [u64; 2],
    /// switch from orderbookonly to init
    pub orderbook_to_init_time: u64,

    /// swap coin in amount
    pub swap_coin_in_amount: u128,
    /// swap pc out amount
    pub swap_pc_out_amount: u128,
    /// charge pc as swap fee while swap pc to coin
    pub swap_acc_pc_fee: u64,

    /// swap pc in amount
    pub swap_pc_in_amount: u128,
    /// swap coin out amount
    pub swap_coin_out_amount: u128,
    /// charge coin as swap fee while swap coin to pc
    pub swap_acc_coin_fee: u64,
}
impl StateData {
    pub fn initialize(&mut self, open_time: u64) -> Result<(), Error> {
        self.need_take_pnl_coin = 0u64;
        self.need_take_pnl_pc = 0u64;
        self.total_pnl_pc = 0u64;
        self.total_pnl_coin = 0u64;
        self.pool_open_time = open_time;
        self.padding = Zeroable::zeroed();
        self.orderbook_to_init_time = 0u64;
        self.swap_coin_in_amount = 0u128;
        self.swap_pc_out_amount = 0u128;
        self.swap_acc_pc_fee = 0u64;
        self.swap_pc_in_amount = 0u128;
        self.swap_coin_out_amount = 0u128;
        self.swap_acc_coin_fee = 0u64;
        Ok(())
    }
}
// https://github.com/raydium-io/raydium-amm/blob/2748852a7981c2b6909e07e10b1325669fbb9195/program/src/state.rs#L42
pub trait Loadable: Pod {
    fn load_mut<'a>(account: &'a AccountInfo) -> Result<RefMut<'a, Self>, ProgramError> {
        // TODO verify if this checks for size
        Ok(RefMut::map(account.try_borrow_mut_data()?, |data| from_bytes_mut(data)))
    }
    fn load<'a>(account: &'a AccountInfo) -> Result<Ref<'a, Self>, ProgramError> {
        Ok(Ref::map(account.try_borrow_data()?, |data| from_bytes(data)))
    }
    fn load_from_bytes(data: &[u8]) -> Result<&Self, ProgramError> {
        Ok(from_bytes(data))
    }
}
// https://github.com/raydium-io/raydium-amm/blob/2748852a7981c2b6909e07e10b1325669fbb9195/program/src/state.rs#L711
unsafe impl Zeroable for AmmInfo {}
// https://github.com/raydium-io/raydium-amm/blob/2748852a7981c2b6909e07e10b1325669fbb9195/program/src/state.rs#L711
unsafe impl Pod for AmmInfo {}
// https://github.com/raydium-io/raydium-amm/blob/2748852a7981c2b6909e07e10b1325669fbb9195/program/src/state.rs#L711
unsafe impl TriviallyTransmutable for AmmInfo {}
// https://github.com/raydium-io/raydium-amm/blob/2748852a7981c2b6909e07e10b1325669fbb9195/program/src/state.rs#L711
impl Loadable for AmmInfo {}
#[repr(u64)]
pub enum AmmStatus {
    Uninitialized = 0u64,
    Initialized = 1u64,
    Disabled = 2u64,
    WithdrawOnly = 3u64,
    // pool only can add or remove liquidity, can't swap and plan orders
    LiquidityOnly = 4u64,
    // pool only can add or remove liquidity and plan orders, can't swap
    OrderBookOnly = 5u64,
    // pool only can add or remove liquidity and swap, can't plan orders
    SwapOnly = 6u64,
    // pool status after created and will auto update to SwapOnly during swap after open_time
    WaitingTrade = 7u64,
}
impl AmmStatus {
    pub fn from_u64(status: u64) -> Self {
        match status {
            0u64 => AmmStatus::Uninitialized,
            1u64 => AmmStatus::Initialized,
            2u64 => AmmStatus::Disabled,
            3u64 => AmmStatus::WithdrawOnly,
            4u64 => AmmStatus::LiquidityOnly,
            5u64 => AmmStatus::OrderBookOnly,
            6u64 => AmmStatus::SwapOnly,
            7u64 => AmmStatus::WaitingTrade,
            _ => unreachable!(),
        }
    }
    pub fn into_u64(&self) -> u64 {
        match self {
            AmmStatus::Uninitialized => 0u64,
            AmmStatus::Initialized => 1u64,
            AmmStatus::Disabled => 2u64,
            AmmStatus::WithdrawOnly => 3u64,
            AmmStatus::LiquidityOnly => 4u64,
            AmmStatus::OrderBookOnly => 5u64,
            AmmStatus::SwapOnly => 6u64,
            AmmStatus::WaitingTrade => 7u64,
        }
    }
    pub fn valid_status(status: u64) -> bool {
        match status {
            1u64 | 2u64 | 3u64 | 4u64 | 5u64 | 6u64 | 7u64 => true,
            _ => false,
        }
    }
    pub fn deposit_permission(&self) -> bool {
        match self {
            AmmStatus::Uninitialized => false,
            AmmStatus::Initialized => true,
            AmmStatus::Disabled => false,
            AmmStatus::WithdrawOnly => false,
            AmmStatus::LiquidityOnly => true,
            AmmStatus::OrderBookOnly => true,
            AmmStatus::SwapOnly => true,
            AmmStatus::WaitingTrade => true,
        }
    }
    pub fn withdraw_permission(&self) -> bool {
        match self {
            AmmStatus::Uninitialized => false,
            AmmStatus::Initialized => true,
            AmmStatus::Disabled => false,
            AmmStatus::WithdrawOnly => true,
            AmmStatus::LiquidityOnly => true,
            AmmStatus::OrderBookOnly => true,
            AmmStatus::SwapOnly => true,
            AmmStatus::WaitingTrade => true,
        }
    }
    pub fn swap_permission(&self) -> bool {
        match self {
            AmmStatus::Uninitialized => false,
            AmmStatus::Initialized => true,
            AmmStatus::Disabled => false,
            AmmStatus::WithdrawOnly => false,
            AmmStatus::LiquidityOnly => false,
            AmmStatus::OrderBookOnly => false,
            AmmStatus::SwapOnly => true,
            AmmStatus::WaitingTrade => true,
        }
    }
    pub fn orderbook_permission(&self) -> bool {
        match self {
            AmmStatus::Uninitialized => false,
            AmmStatus::Initialized => true,
            AmmStatus::Disabled => false,
            AmmStatus::WithdrawOnly => false,
            AmmStatus::LiquidityOnly => false,
            AmmStatus::OrderBookOnly => true,
            AmmStatus::SwapOnly => false,
            AmmStatus::WaitingTrade => false,
        }
    }
}
// https://github.com/raydium-io/raydium-amm/blob/2748852a7981c2b6909e07e10b1325669fbb9195/program/src/state.rs#L475
#[repr(
    C,
    packed
)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Fees {
    /// numerator of the min_separate
    pub min_separate_numerator: u64,
    /// denominator of the min_separate
    pub min_separate_denominator: u64,

    /// numerator of the fee
    pub trade_fee_numerator: u64,
    /// denominator of the fee
    /// and 'trade_fee_denominator' must be equal to 'min_separate_denominator'
    pub trade_fee_denominator: u64,

    /// numerator of the pnl
    pub pnl_numerator: u64,
    /// denominator of the pnl
    pub pnl_denominator: u64,

    /// numerator of the swap_fee
    pub swap_fee_numerator: u64,
    /// denominator of the swap_fee
    pub swap_fee_denominator: u64,
}
// https://github.com/raydium-io/raydium-amm/blob/2748852a7981c2b6909e07e10b1325669fbb9195/program/src/state.rs#L465
fn validate_fraction(numerator: u64, denominator: u64) -> Result<(), Error> {
    if numerator >= denominator || denominator == 0 {
        Err(Error::InvalidFee)
    } else {
        Ok(())
    }
}
// https://github.com/raydium-io/raydium-amm/blob/2748852a7981c2b6909e07e10b1325669fbb9195/program/src/state.rs#L498
impl Fees {
    /// Validate that the fees are reasonable
    pub fn validate(&self) -> Result<(), Error> {
        validate_fraction(self.min_separate_numerator, self.min_separate_denominator)?;
        validate_fraction(self.trade_fee_numerator, self.trade_fee_denominator)?;
        validate_fraction(self.pnl_numerator, self.pnl_denominator)?;
        validate_fraction(self.swap_fee_numerator, self.swap_fee_denominator)?;
        Ok(())
    }
    pub fn initialize(&mut self) -> Result<(), Error> {
        // min_separate = 5/10000
        self.min_separate_numerator = 5;
        self.min_separate_denominator = TEN_THOUSAND;
        // trade_fee = 25/10000
        self.trade_fee_numerator = 25;
        self.trade_fee_denominator = TEN_THOUSAND;
        // pnl = 12/100
        self.pnl_numerator = 12;
        self.pnl_denominator = 100;
        // swap_fee = 25 / 10000
        self.swap_fee_numerator = 25;
        self.swap_fee_denominator = TEN_THOUSAND;
        Ok(())
    }
}
// https://github.com/raydium-io/raydium-amm/blob/2748852a7981c2b6909e07e10b1325669fbb9195/program/src/state.rs#L526
impl IsInitialized for Fees {
    fn is_initialized(&self) -> bool {
        true
    }
}
pub const TEN_THOUSAND: u64 = 10000;
// https://github.com/raydium-io/raydium-amm/blob/2748852a7981c2b6909e07e10b1325669fbb9195/program/src/state.rs#L532
impl Sealed for Fees {}
// https://github.com/raydium-io/raydium-amm/blob/2748852a7981c2b6909e07e10b1325669fbb9195/program/src/state.rs#L533
impl Pack for Fees {
    const LEN: usize = 64;
    fn pack_into_slice(&self, output: &mut [u8]) {
        let output = array_mut_ref![output, 0, 64];
        let (
            min_separate_numerator,
            min_separate_denominator,
            trade_fee_numerator,
            trade_fee_denominator,
            pnl_numerator,
            pnl_denominator,
            swap_fee_numerator,
            swap_fee_denominator,
        ) = mut_array_refs![output, 8, 8, 8, 8, 8, 8, 8, 8];
        *min_separate_numerator = self.min_separate_numerator.to_le_bytes();
        *min_separate_denominator = self.min_separate_denominator.to_le_bytes();
        *trade_fee_numerator = self.trade_fee_numerator.to_le_bytes();
        *trade_fee_denominator = self.trade_fee_denominator.to_le_bytes();
        *pnl_numerator = self.pnl_numerator.to_le_bytes();
        *pnl_denominator = self.pnl_denominator.to_le_bytes();
        *swap_fee_numerator = self.swap_fee_numerator.to_le_bytes();
        *swap_fee_denominator = self.swap_fee_denominator.to_le_bytes();
    }
    fn unpack_from_slice(input: &[u8]) -> Result<Fees, ProgramError> {
        let input = array_ref![input, 0, 64];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            min_separate_numerator,
            min_separate_denominator,
            trade_fee_numerator,
            trade_fee_denominator,
            pnl_numerator,
            pnl_denominator,
            swap_fee_numerator,
            swap_fee_denominator,
        ) = array_refs![input, 8, 8, 8, 8, 8, 8, 8, 8];
        Ok(Self {
            min_separate_numerator: u64::from_le_bytes(*min_separate_numerator),
            min_separate_denominator: u64::from_le_bytes(*min_separate_denominator),
            trade_fee_numerator: u64::from_le_bytes(*trade_fee_numerator),
            trade_fee_denominator: u64::from_le_bytes(*trade_fee_denominator),
            pnl_numerator: u64::from_le_bytes(*pnl_numerator),
            pnl_denominator: u64::from_le_bytes(*pnl_denominator),
            swap_fee_numerator: u64::from_le_bytes(*swap_fee_numerator),
            swap_fee_denominator: u64::from_le_bytes(*swap_fee_denominator),
        })
    }
}
pub fn unpack_token_account(account_info: &AccountInfo, token_program_id: &Pubkey) -> Result<Account, Error> {
    if account_info.owner != token_program_id {
        Err(Error::InvalidSplTokenProgram)
    } else {
        Account::unpack(&account_info.data.borrow()).map_err(|_| Error::ExpectedAccount)
    }
}
#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct MarketState {
    // 0
    pub account_flags: u64, // Initialized, Market

    // 1
    pub own_address: [u64; 4],

    // 5
    pub vault_signer_nonce: u64,
    // 6
    pub coin_mint: [u64; 4],
    // 10
    pub pc_mint: [u64; 4],

    // 14
    pub coin_vault: [u64; 4],
    // 18
    pub coin_deposits_total: u64,
    // 19
    pub coin_fees_accrued: u64,

    // 20
    pub pc_vault: [u64; 4],
    // 24
    pub pc_deposits_total: u64,
    // 25
    pub pc_fees_accrued: u64,

    // 26
    pub pc_dust_threshold: u64,

    // 27
    pub req_q: [u64; 4],
    // 31
    pub event_q: [u64; 4],

    // 35
    pub bids: [u64; 4],
    // 39
    pub asks: [u64; 4],

    // 43
    pub coin_lot_size: u64,
    // 44
    pub pc_lot_size: u64,

    // 45
    pub fee_rate_bps: u64,
    // 46
    pub referrer_rebates_accrued: u64,
}
unsafe impl Zeroable for MarketState {}
unsafe impl Pod for MarketState {}
unsafe impl TriviallyTransmutable for MarketState {}
impl MarketState {
    pub fn load_event_queue_mut<'a>(&'a self, queue: &'a AccountInfo) -> Result<EventQueue<'a>, Error> {
        if queue.key.to_aligned_bytes() != identity(self.event_q) {
            return Err(Error::WrongEventQueueAccount);
        }
        let (header, buf) = strip_header::<EventQueueHeader, Event>(queue, false)?;
        let flags = BitFlags::from_bits(header.account_flags).unwrap();
        if flags != (AccountFlag::Initialized | AccountFlag::EventQueue) {
            return Err(Error::InvalidLogic);
        }
        Ok(Queue {
            header,
            buf,
        })
    }
}
#[derive(Copy, Clone, BitFlags, Debug, Eq, PartialEq)]
#[repr(u64)]
pub enum AccountFlag {
    Initialized = 1u64 << 0,
    Market = 1u64 << 1,
    OpenOrders = 1u64 << 2,
    RequestQueue = 1u64 << 3,
    EventQueue = 1u64 << 4,
    Bids = 1u64 << 5,
    Asks = 1u64 << 6,
    Disabled = 1u64 << 7,
    Closed = 1u64 << 8,
    Permissioned = 1u64 << 9,
    CrankAuthorityRequired = 1u64 << 10,
}
pub struct Queue<'a, H: QueueHeader> {
    header: RefMut<'a, H>,
    buf: RefMut<'a, [H::Item]>,
}
#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct EventQueueHeader {
    account_flags: u64, // Initialized, EventQueue
    head: u64,
    count: u64,
    seq_num: u64,
}
unsafe impl Zeroable for EventQueueHeader {}
unsafe impl Pod for EventQueueHeader {}
unsafe impl TriviallyTransmutable for EventQueueHeader {}
unsafe impl TriviallyTransmutable for RequestQueueHeader {}
pub trait QueueHeader: Pod {
    type Item: Pod + Copy;
    fn head(&self) -> u64;
    fn set_head(&mut self, value: u64);
    fn count(&self) -> u64;
    fn set_count(&mut self, value: u64);
    fn incr_event_id(&mut self);
    fn decr_event_id(&mut self, n: u64);
}
impl QueueHeader for EventQueueHeader {
    type Item = Event;
    fn head(&self) -> u64 {
        self.head
    }
    fn set_head(&mut self, value: u64) {
        self.head = value;
    }
    fn count(&self) -> u64 {
        self.count
    }
    fn set_count(&mut self, value: u64) {
        self.count = value;
    }
    fn incr_event_id(&mut self) {
        self.seq_num += 1;
    }
    fn decr_event_id(&mut self, n: u64) {
        self.seq_num -= n;
    }
}
impl<'a, H: QueueHeader> Queue<'a, H> {
    pub fn new(header: RefMut<'a, H>, buf: RefMut<'a, [H::Item]>) -> Self {
        Self {
            header,
            buf,
        }
    }
    #[inline]
    pub fn len(&self) -> u64 {
        self.header.count()
    }
    #[inline]
    pub fn full(&self) -> bool {
        self.header.count() as usize == self.buf.len()
    }
    #[inline]
    pub fn empty(&self) -> bool {
        self.header.count() == 0
    }
    #[inline]
    pub fn push_back(&mut self, value: H::Item) -> Result<(), H::Item> {
        if self.full() {
            return Err(value);
        }
        let slot = ((self.header.head() + self.header.count()) as usize) % self.buf.len();
        self.buf[slot] = value;
        let count = self.header.count();
        self.header.set_count(count + 1);
        self.header.incr_event_id();
        Ok(())
    }
    #[inline]
    pub fn peek_front(&self) -> Option<&H::Item> {
        if self.empty() {
            return None;
        }
        Some(&self.buf[self.header.head() as usize])
    }
    #[inline]
    pub fn peek_front_mut(&mut self) -> Option<&mut H::Item> {
        if self.empty() {
            return None;
        }
        Some(&mut self.buf[self.header.head() as usize])
    }
    #[inline]
    pub fn pop_front(&mut self) -> Result<H::Item, ()> {
        if self.empty() {
            return Err(());
        }
        let value = self.buf[self.header.head() as usize];
        let count = self.header.count();
        self.header.set_count(count - 1);
        let head = self.header.head();
        self.header.set_head((head + 1) % self.buf.len() as u64);
        Ok(value)
    }
    #[inline]
    pub fn revert_pushes(&mut self, desired_len: u64) -> Result<(), Error> {
        if desired_len > self.header.count() {
            return Err(Error::InvalidLogic);
        }
        let len_diff = self.header.count() - desired_len;
        self.header.set_count(desired_len);
        self.header.decr_event_id(len_diff);
        Ok(())
    }
    pub fn iter(&self) -> impl Iterator<Item = &H::Item> {
        QueueIterator {
            queue: self,
            index: 0,
        }
    }
}
struct QueueIterator<'a, 'b, H: QueueHeader> {
    queue: &'b Queue<'a, H>,
    index: u64,
}
impl<'a, 'b, H: QueueHeader> Iterator for QueueIterator<'a, 'b, H> {
    type Item = &'b H::Item;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.queue.len() {
            None
        } else {
            let item = &self.queue.buf[(self.queue.header.head() + self.index) as usize % self.queue.buf.len()];
            self.index += 1;
            Some(item)
        }
    }
}
#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct RequestQueueHeader {
    account_flags: u64, // Initialized, RequestQueue
    head: u64,
    count: u64,
    next_seq_num: u64,
}
unsafe impl Zeroable for RequestQueueHeader {}
unsafe impl Pod for RequestQueueHeader {}
impl QueueHeader for RequestQueueHeader {
    type Item = Request;
    fn head(&self) -> u64 {
        self.head
    }
    fn set_head(&mut self, value: u64) {
        self.head = value;
    }
    fn count(&self) -> u64 {
        self.count
    }
    fn set_count(&mut self, value: u64) {
        self.count = value;
    }
    #[inline(always)]
    fn incr_event_id(&mut self) {}
    #[inline(always)]
    fn decr_event_id(&mut self, _n: u64) {}
}
#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct Request {
    request_flags: u8,
    owner_slot: u8,
    fee_tier: u8,
    self_trade_behavior: u8,
    padding: [u8; 4],
    max_coin_qty_or_cancel_id: u64,
    native_pc_qty_locked: u64,
    order_id: u128,
    owner: [u64; 4],
    client_order_id: u64,
}
unsafe impl Zeroable for Request {}
unsafe impl Pod for Request {}
pub type RequestQueue<'a> = Queue<'a, RequestQueueHeader>;
pub type EventQueue<'a> = Queue<'a, EventQueueHeader>;
#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct Event {
    event_flags: u8,
    owner_slot: u8,

    fee_tier: u8,

    _padding: [u8; 5],

    native_qty_released: u64,
    native_qty_paid: u64,
    native_fee_or_rebate: u64,

    order_id: u128,
    pub owner: [u64; 4],
    client_order_id: u64,
}
unsafe impl Zeroable for Event {}
unsafe impl Pod for Event {}
unsafe impl TriviallyTransmutable for Event {}
impl Event {
    #[inline(always)]
    pub fn as_view(&self) -> Result<EventView, Error> {
        let flags = BitFlags::from_bits(self.event_flags).unwrap();
        let side = EventFlag::flags_to_side(flags);
        let client_order_id = NonZeroU64::new(self.client_order_id);
        if flags.contains(EventFlag::Fill) {
            let allowed_flags = {
                use EventFlag::*;
                Fill | Bid | Maker
            };
            if !allowed_flags.contains(flags) {
                return Err(Error::InvalidLogic);
            }
            return Ok(EventView::Fill {
                side,
                maker: flags.contains(EventFlag::Maker),
                native_qty_paid: self.native_qty_paid,
                native_qty_received: self.native_qty_released,
                native_fee_or_rebate: self.native_fee_or_rebate,

                order_id: self.order_id,
                owner: self.owner,

                owner_slot: self.owner_slot,
                fee_tier: self.fee_tier.try_into().map_err(|_| Error::InvalidLogic)?,
                client_order_id,
            });
        }
        let allowed_flags = {
            use EventFlag::*;
            Out | Bid | ReleaseFunds
        };
        if !allowed_flags.contains(flags) {
            return Err(Error::InvalidLogic);
        }
        Ok(EventView::Out {
            side,
            release_funds: flags.contains(EventFlag::ReleaseFunds),
            native_qty_unlocked: self.native_qty_released,
            native_qty_still_locked: self.native_qty_paid,

            order_id: self.order_id,
            owner: self.owner,

            owner_slot: self.owner_slot,
            client_order_id,
        })
    }
}
#[derive(Copy, Clone, BitFlags, Debug)]
#[repr(u8)]
enum EventFlag {
    Fill = 0x1,
    Out = 0x2,
    Bid = 0x4,
    Maker = 0x8,
    ReleaseFunds = 0x10,
}
impl EventFlag {
    #[inline]
    fn from_side(side: Side) -> BitFlags<Self> {
        match side {
            Side::Bid => EventFlag::Bid.into(),
            Side::Ask => BitFlags::empty(),
        }
    }
    #[inline]
    fn flags_to_side(flags: BitFlags<Self>) -> Side {
        if flags.contains(EventFlag::Bid) {
            Side::Bid
        } else {
            Side::Ask
        }
    }
}
#[derive(Debug)]
pub enum EventView {
    Fill {
        side: Side,
        maker: bool,
        native_qty_paid: u64,
        native_qty_received: u64,
        native_fee_or_rebate: u64,
        order_id: u128,
        owner: [u64; 4],
        owner_slot: u8,
        fee_tier: FeeTier,
        client_order_id: Option<NonZeroU64>,
    },
    Out {
        side: Side,
        release_funds: bool,
        native_qty_unlocked: u64,
        native_qty_still_locked: u64,
        order_id: u128,
        owner: [u64; 4],
        owner_slot: u8,
        client_order_id: Option<NonZeroU64>,
    },
}
impl EventView {
    fn side(&self) -> Side {
        match self {
            &EventView::Fill {
                side,
                ..
            }
            | &EventView::Out {
                side,
                ..
            } => side,
        }
    }
}
#[derive(Copy, Clone, IntoPrimitive, TryFromPrimitive, Debug)]
#[repr(u8)]
pub enum FeeTier {
    Base,
    _SRM2,
    _SRM3,
    _SRM4,
    _SRM5,
    _SRM6,
    _MSRM,
    Stable,
}
#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct OpenOrders {
    pub account_flags: u64, // Initialized, OpenOrders
    pub market: [u64; 4],
    pub owner: [u64; 4],

    pub native_coin_free: u64,
    pub native_coin_total: u64,

    pub native_pc_free: u64,
    pub native_pc_total: u64,

    pub free_slot_bits: u128,
    pub is_bid_bits: u128,
    pub orders: [u128; 128],
    // Using Option<NonZeroU64> in a pod type requires nightly
    pub client_order_ids: [u64; 128],
    pub referrer_rebates_accrued: u64,
}
unsafe impl Pod for OpenOrders {}
unsafe impl Zeroable for OpenOrders {}
impl OpenOrders {
    fn check_flags(&self) -> Result<(), Error> {
        let flags = BitFlags::from_bits(self.account_flags).map_err(|_| Error::InvalidLogic)?;
        let required_flags = AccountFlag::Initialized | AccountFlag::OpenOrders;
        if flags != required_flags {
            Err(Error::InvalidLogic)?
        }
        Ok(())
    }
    #[inline]
    pub fn load_checked<'a>(
        orders_account: &'a AccountInfo,
        market_account: Option<&AccountInfo>,
        owner_account: Option<&AccountInfo>,
        program_id: &Pubkey,
    ) -> Result<Ref<'a, Self>, Error> {
        if orders_account.owner != program_id {
            return Err(Error::InvalidLogic);
        }
        let account_data = orders_account.try_borrow_data().map_err(|_| Error::InvalidLogic)?;
        if account_data.len() < 12 {
            return Err(Error::InvalidLogic);
        }
        let head = array_ref![account_data, 0, 5];
        let tail = array_ref![account_data, account_data.len() - 7, 7];
        if head != ACCOUNT_HEAD_PADDING {
            return Err(Error::InvalidLogic);
        }
        if tail != ACCOUNT_TAIL_PADDING {
            return Err(Error::InvalidLogic);
        }
        let state: Ref<'a, Self> = Ref::map(account_data, |account_data| bytemuck::from_bytes(&account_data[5..account_data.len() - 7]));
        state.check_flags()?;
        if let Some(market_acc) = market_account {
            if identity(state.market) != market_acc.key.to_aligned_bytes() {
                return Err(Error::InvalidLogic);
            }
        }
        if let Some(owner) = owner_account {
            if identity(state.owner) != owner.key.to_aligned_bytes() {
                return Err(Error::InvalidLogic);
            }
        }
        Ok(state)
    }
}
pub trait ToAlignedBytes {
    fn to_aligned_bytes(&self) -> [u64; 4];
}
impl ToAlignedBytes for Pubkey {
    #[inline]
    fn to_aligned_bytes(&self) -> [u64; 4] {
        cast(self.to_bytes())
    }
}
#[inline]
fn remove_slop_mut<T: Pod>(bytes: &mut [u8]) -> &mut [T] {
    let slop = bytes.len() % size_of::<T>();
    let new_len = bytes.len() - slop;
    cast_slice_mut(&mut bytes[..new_len])
}
fn strip_account_padding(padded_data: &mut [u8], init_allowed: bool) -> Result<&mut [[u8; 8]], Error> {
    if init_allowed {
        init_account_padding(padded_data)
    } else {
        check_account_padding(padded_data)
    }
}
#[derive(Eq, PartialEq, Copy, Clone, TryFromPrimitive, IntoPrimitive, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Side {
    Bid = 0,
    Ask = 1,
}
pub fn strip_header<'a, H: Pod, D: Pod>(account: &'a AccountInfo, init_allowed: bool) -> Result<(RefMut<'a, H>, RefMut<'a, [D]>), Error> {
    let mut result = Ok(());
    let account_ = account.try_borrow_mut_data().map_err(|_| Error::InvalidLogic)?;
    let (header, inner): (RefMut<'a, [H]>, RefMut<'a, [D]>) = RefMut::map_split(account_, |padded_data| {
        let dummy_value: (&mut [H], &mut [D]) = (&mut [], &mut []);
        let padded_data: &mut [u8] = *padded_data;
        let u64_data = match strip_account_padding(padded_data, init_allowed) {
            Ok(u64_data) => u64_data,
            Err(e) => {
                result = Err(e);
                return dummy_value;
            }
        };
        let data: &mut [u8] = cast_slice_mut(u64_data);
        let (header_bytes, inner_bytes) = data.split_at_mut(size_of::<H>());
        let header: &mut H;
        let inner: &mut [D];
        header = match try_from_bytes_mut(header_bytes) {
            Ok(h) => h,
            Err(_e) => {
                result = Err(Error::InvalidLogic);
                return dummy_value;
            }
        };
        inner = remove_slop_mut(inner_bytes);
        (std::slice::from_mut(header), inner)
    });
    result?;
    let header = RefMut::map(header, |s| s.first_mut().unwrap_or_else(|| unreachable!()));
    Ok((header, inner))
}
fn init_account_padding(data: &mut [u8]) -> Result<&mut [[u8; 8]], Error> {
    if data.len() < 12 {
        return Err(Error::InvalidLogic);
    }
    let (head, data, tail) = mut_array_refs![data, 5; ..; 7];
    *head = *ACCOUNT_HEAD_PADDING;
    *tail = *ACCOUNT_TAIL_PADDING;
    try_cast_slice_mut(data).map_err(|_| Error::InvalidLogic)
}
fn check_account_padding(data: &mut [u8]) -> Result<&mut [[u8; 8]], Error> {
    if data.len() < 12 {
        return Err(Error::InvalidLogic);
    }
    let (head, data, tail) = mut_array_refs![data, 5; ..; 7];
    if head != ACCOUNT_HEAD_PADDING {
        return Err(Error::InvalidLogic);
    }
    if tail != ACCOUNT_TAIL_PADDING {
        return Err(Error::InvalidLogic);
    }
    try_cast_slice_mut(data).map_err(|_| Error::InvalidLogic)
}
pub fn calc_total_without_take_pnl_no_orderbook(pc_amount: u64, coin_amount: u64, amm: &AmmInfo) -> Result<(u64, u64), ProgramError> {
    let total_pc_without_take_pnl = pc_amount.checked_sub(amm.state_data.need_take_pnl_pc).ok_or(ProgramError::ArithmeticOverflow)?;
    let total_coin_without_take_pnl = coin_amount.checked_sub(amm.state_data.need_take_pnl_coin).ok_or(ProgramError::ArithmeticOverflow)?;
    Ok((total_pc_without_take_pnl, total_coin_without_take_pnl))
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u64)]
pub enum SwapDirection {
    /// Input token pc, output token coin
    PC2Coin = 1u64,
    /// Input token coin, output token pc
    Coin2PC = 2u64,
}
construct_uint! {
    pub struct U128(2);
}
impl CheckedCeilDiv for U128 {
    fn checked_ceil_div(&self, mut rhs: Self) -> Option<(Self, Self)> {
        let mut quotient = self.checked_div(rhs)?;
        // Avoid dividing a small number by a big one and returning 1, and instead
        // fail.
        let zero = U128::from(0);
        let one = U128::from(1);
        if quotient.is_zero() {
            // return None;
            if self.checked_mul(U128::from(2))? >= rhs {
                return Some((one, zero));
            } else {
                return Some((zero, zero));
            }
        }
        // Ceiling the destination amount if there's any remainder, which will
        // almost always be the case.
        let remainder = self.checked_rem(rhs)?;
        if remainder > zero {
            quotient = quotient.checked_add(one)?;
            // calculate the minimum amount needed to get the dividend amount to
            // avoid truncating too much
            rhs = self.checked_div(quotient)?;
            let remainder = self.checked_rem(quotient)?;
            if remainder > zero {
                rhs = rhs.checked_add(one)?;
            }
        }
        Some((quotient, rhs))
    }
}
pub fn swap_token_amount_base_in(amount_in: U128, total_pc_without_take_pnl: U128, total_coin_without_take_pnl: U128, swap_direction: SwapDirection) -> U128 {
    match swap_direction {
        SwapDirection::Coin2PC => {
            // (x + delta_x) * (y + delta_y) = x * y
            // (coin + amount_in) * (pc - amount_out) = coin * pc
            // => amount_out = pc - coin * pc / (coin + amount_in)
            // => amount_out = ((pc * coin + pc * amount_in) - coin * pc) / (coin + amount_in)
            // => amount_out =  pc * amount_in / (coin + amount_in)
            let denominator = total_coin_without_take_pnl.checked_add(amount_in).unwrap();
            total_pc_without_take_pnl.checked_mul(amount_in).unwrap().checked_div(denominator).unwrap()
        }
        SwapDirection::PC2Coin => {
            // (x + delta_x) * (y + delta_y) = x * y
            // (pc + amount_in) * (coin - amount_out) = coin * pc
            // => amount_out = coin - coin * pc / (pc + amount_in)
            // => amount_out = (coin * pc + coin * amount_in - coin * pc) / (pc + amount_in)
            // => amount_out = coin * amount_in / (pc + amount_in)
            let denominator = total_pc_without_take_pnl.checked_add(amount_in).unwrap();
            total_coin_without_take_pnl.checked_mul(amount_in).unwrap().checked_div(denominator).unwrap()
        }
    }
}
