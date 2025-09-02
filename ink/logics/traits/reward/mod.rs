pub mod psp22_reward;

use inkv5_client_lib::traits::access_control::AccessControlError;

#[derive(Debug, Eq, PartialEq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
#[allow(clippy::cast_possible_truncation)]
pub enum RewardError {
    InsufficientTransferredBalance,
    TransferError,
    AddOverFlow,
    NoReward,
    AccessControlError(AccessControlError),
}

/// convertor from AccessControlError to ParticipantFilterError
impl From<AccessControlError> for RewardError {
    fn from(error: AccessControlError) -> Self {
        RewardError::AccessControlError(error)
    }
}