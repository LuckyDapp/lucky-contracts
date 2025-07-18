use inkv5_client_lib::traits::access_control::AccessControlError;
use inkv5_client_lib::traits::RollupClientError;

#[derive(Debug, Eq, PartialEq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[allow(clippy::cast_possible_truncation)]
pub enum RaffleError {
    IncorrectEra,
    NoReward,
    NoRatioSet,
    IncorrectRatio,
    NoWinner,
    TooManyWinners,
    DivByZero,
    MulOverFlow,
    AddOverFlow,
    AccessControlError(AccessControlError),
    RollupClientError(RollupClientError),
    FailedToDecode,
    TryFromIntError,
    CrossContractCallError1,
    CrossContractCallError2,
    TransferError,
    DappsStakingDeveloperAddressMissing,
    RewardManagerAddressMissing,
}

/// convertor from AccessControlError to ParticipantFilterError
impl From<AccessControlError> for RaffleError {
    fn from(error: AccessControlError) -> Self {
        RaffleError::AccessControlError(error)
    }
}

/// convertor from RollupClientError to ContractError
impl From<RollupClientError> for RaffleError {
    fn from(error: RollupClientError) -> Self {
        RaffleError::RollupClientError(error)
    }
}

/// convertor from ContractError to RollupClientError
impl From<RaffleError> for RollupClientError {
    fn from(error: RaffleError) -> Self {
        ink::env::debug_println!("Error: {:?}", error);
        RollupClientError::UnsupportedAction
    }
}

/// convertor from AccessControlError to ParticipantFilterError
impl From<core::num::TryFromIntError> for RaffleError {
    fn from(_error: core::num::TryFromIntError) -> Self {
        RaffleError::TryFromIntError
    }
}



