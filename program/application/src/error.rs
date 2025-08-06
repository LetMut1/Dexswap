use {
    num_traits::FromPrimitive,
    solana_program_error::{
        ProgramError,
        ToStr,
    },
    std::{
        convert::TryFrom,
        error::Error as Error_,
        fmt::{
            Display,
            Formatter,
            Result as FmtResult,
        },
    },
};
#[derive(Debug, num_derive::FromPrimitive)]
#[repr(u32)]
pub enum Error {
    IntermediaryInvalidAuthority,
    IntermediaryInvalidManager,
    IntermediaryInvalidTemporaryWSolTokenAccount,
    IntermediaryInvalidTrader,
    IntermediaryInvalidWSolTokenAccount,
    IntermediaryIsNotInitialized,
    InvalidAccountConfigurationFlags,
    InvalidAccountData,
    InvalidAccountLamports,
    InvalidAccountPubkey,
    InvalidLogic,
    NotImplemented,
    RepeatableDex,
    EqualMints,
    ZeroAmountIn,
    ZeroDexesPresented,
    InvalidSwapConditions,
    InvalidTokenMint,
    TokenAccountInsufficientAmount,
    TokenAccountInvalidAmount,
    ExpectedAccount,
    InvalidOpenOrders,
    InvalidMarket,
    InvalidOwner,
    InvalidStatus,
    InvalidAmmAccountOwner,
    CheckedSubOverflow,
    CheckedAddOverflow,
    InvalidSplTokenProgram,
    InvalidUserToken,
    InvalidFee,
    WrongEventQueueAccount,
}
impl Display for Error {
    fn fmt(&self, _: &mut Formatter<'_>) -> FmtResult {
        Ok(())
    }
}
impl Error_ for Error {}
impl From<Error> for ProgramError {
    fn from(error: Error) -> Self {
        ProgramError::Custom(error as u32)
    }
}
impl TryFrom<u32> for Error {
    type Error = &'static str;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Error::from_u32(value).ok_or("Non-existent error, it is needed to implement 'TryFrom<u32>' correctly.")
    }
}
impl ToStr for Error {
    fn to_str<E>(&self) -> &'static str
    where
        E: 'static + ToStr + TryFrom<u32>,
    {
        match self {
            Self::IntermediaryInvalidAuthority => "IntermediaryInvalidAuthority",
            Self::IntermediaryInvalidManager => "IntermediaryInvalidManager",
            Self::IntermediaryInvalidTemporaryWSolTokenAccount => "IntermediaryInvalidTemporaryWSolTokenAccount",
            Self::IntermediaryInvalidTrader => "IntermediaryInvalidTrader",
            Self::IntermediaryInvalidWSolTokenAccount => "IntermediaryInvalidWSolTokenAccount",
            Self::IntermediaryIsNotInitialized => "IntermediaryIsNotInitialized",
            Self::InvalidAccountConfigurationFlags => "InvalidAccountConfigurationFlags",
            Self::InvalidAccountData => "InvalidAccountData",
            Self::InvalidAccountLamports => "InvalidAccountLamports",
            Self::InvalidAccountPubkey => "InvalidAccountPubkey",
            Self::InvalidLogic => "InvalidLogic",
            Self::InvalidSwapConditions => "InvalidSwapConditions",
            Self::InvalidTokenMint => "InvalidTokenMint",
            Self::NotImplemented => "NotImplemented",
            Self::RepeatableDex => "RepeatableDex",
            Self::EqualMints => "EqualMints",
            Self::ZeroAmountIn => "ZeroAmountIn",
            Self::ZeroDexesPresented => "ZeroDexesPresented",
            Self::TokenAccountInsufficientAmount => "TokenAccountInsufficientAmount",
            Self::TokenAccountInvalidAmount => "TokenAccountInvalidAmount",
            Self::ExpectedAccount => "ExpectedAccount",
            Self::InvalidOpenOrders => "InvalidOpenOrders",
            Self::InvalidMarket => "InvalidMarket",
            Self::InvalidOwner => "InvalidOwner",
            Self::InvalidStatus => "InvalidStatus",
            Self::InvalidAmmAccountOwner => "InvalidAmmAccountOwner",
            Self::CheckedSubOverflow => "CheckedSubOverflow",
            Self::CheckedAddOverflow => "CheckedAddOverflow",
            Self::InvalidSplTokenProgram => "InvalidSplTokenProgram",
            Self::InvalidUserToken => "InvalidUserToken",
            Self::InvalidFee => "InvalidFee",
            Self::WrongEventQueueAccount => "WrongEventQueueAccount",
        }
    }
}
