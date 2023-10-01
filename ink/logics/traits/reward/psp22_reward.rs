use crate::traits::reward::psp22_reward::RewardError::*;
use ink::prelude::vec::Vec;
use openbrush::contracts::access_control::{access_control, AccessControlError, RoleType};
use openbrush::storage::Mapping;
use openbrush::traits::{AccountId, Balance, Storage};

#[openbrush::wrapper]
pub type Psp22RewardRef = dyn Psp22Reward;

pub const REWARD_MANAGER: RoleType = ink::selector_id!("REWARD_MANAGER");
pub const REWARD_VIEWER: RoleType = ink::selector_id!("REWARD_VIEWER");

#[derive(Default, Debug)]
#[openbrush::storage_item]
pub struct Data {
    pending_rewards: Mapping<AccountId, Balance>,
}

#[openbrush::trait_definition]
pub trait Psp22Reward: Internal + Storage<Data> + access_control::Internal {
    /// Add the accounts in the list of winners for a given era
    /// accounts contains the list of winners and the rewards by account
    #[ink(message, payable, selector = 0xc218e5ba)]
    #[openbrush::modifiers(access_control::only_role(REWARD_MANAGER))]
    fn fund_rewards_and_add_winners(
        &mut self,
        era: u32,
        accounts: Vec<(AccountId, Balance)>,
    ) -> Result<(), RewardError> {
        let transferred_value = Self::env().transferred_value();
        let mut total_rewards = Balance::default();

        // iterate on the accounts (the winners)
        for (account, reward) in accounts {
            total_rewards = total_rewards.checked_add(reward).ok_or(AddOverFlow)?;

            // compute the new rewards for this winner
            let new_reward = match self.data::<Data>().pending_rewards.get(&account) {
                Some(existing_reward) => existing_reward.checked_add(reward).ok_or(AddOverFlow)?,
                _ => reward,
            };

            // add the pending rewards for this account
            self.data::<Data>()
                .pending_rewards
                .insert(&account, &new_reward);

            self._emit_pending_reward_event(account, era, reward);
        }

        if transferred_value < total_rewards {
            return Err(InsufficientTransferredBalance);
        }

        Ok(())
    }

    /// return true if the current account has pending rewards
    #[ink(message)]
    fn has_pending_rewards(&self) -> bool {
        let from = Self::env().caller();
        self._has_pending_rewards_from(from)
    }

    fn _has_pending_rewards_from(&self, from: AccountId) -> bool {
        self.data::<Data>().pending_rewards.contains(&from)
    }

    /// return the pending rewards for a given account.
    #[ink(message)]
    #[openbrush::modifiers(access_control::only_role(REWARD_VIEWER))]
    fn get_pending_rewards_from(
        &mut self,
        from: AccountId,
    ) -> Result<Option<Balance>, RewardError> {
        Ok(self.data::<Data>().pending_rewards.get(&from))
    }

    /// claim all pending rewards for the current account
    /// After claiming, there is not anymore pending rewards for this account
    #[ink(message)]
    fn claim(&mut self) -> Result<(), RewardError> {
        let from = Self::env().caller();
        self._claim_from(from)
    }

    /// claim all pending rewards for the given account
    /// After claiming, there is not anymore pending rewards for this account
    fn _claim_from(&mut self, from: AccountId) -> Result<(), RewardError> {
        // get all pending rewards for this account
        match self.data::<Data>().pending_rewards.get(&from) {
            Some(pending_rewards) => {
                // transfer the amount
                Self::env()
                    .transfer(from, pending_rewards)
                    .map_err(|_| TransferError)?;
                // emmit the event
                self._emit_rewards_claimed_event(from, pending_rewards);
                // remove the pending rewards
                self.data::<Data>().pending_rewards.remove(&from);
                Ok(())
            }
            _ => Err(NoReward),
        }
    }
}

#[openbrush::trait_definition]
pub trait Internal {
    fn _emit_pending_reward_event(&self, account: AccountId, era: u32, amount: Balance);
    fn _emit_rewards_claimed_event(&self, account: AccountId, amount: Balance);
}

#[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RewardError {
    InsufficientTransferredBalance,
    TransferError,
    AddOverFlow,
    NoReward,
    AccessControlError(AccessControlError),
}

/// convertor from AccessControlError to RewardError
impl From<AccessControlError> for RewardError {
    fn from(error: AccessControlError) -> Self {
        RewardError::AccessControlError(error)
    }
}
