use crate::traits::reward::RewardError;
use ink::prelude::vec::Vec;
use ink::storage::Mapping;
use ink::primitives::AccountId;
use inkv5_client_lib::traits::access_control::{BaseAccessControl};
use ink::env::DefaultEnvironment;
use crate::traits::Balance;
/*
#[openbrush::wrapper]
pub type Psp22RewardRef = dyn Psp22Reward;
 */

pub const REWARD_MANAGER_ROLE: u32 = ink::selector_id!("REWARD_MANAGER");
pub const REWARD_VIEWER_ROLE: u32 = ink::selector_id!("REWARD_VIEWER");


#[derive(Default, Debug)]
#[ink::storage_item]
pub struct Psp22RewardData {
    pending_rewards: Mapping<AccountId, Balance>,
}


/// Event emitted when a reward is pending
#[ink::event]
pub struct PendingReward {
    #[ink(topic)]
    account: AccountId,
    era: u32,
    amount: Balance,
}

/// Event emitted when a user claim rewards
#[ink::event]
pub struct RewardsClaimed {
    #[ink(topic)]
    account: AccountId,
    amount: Balance,
}


#[ink::trait_definition]
pub trait Psp22Reward {
    /// Add the accounts in the list of winners for a given era
    /// accounts contains the list of winners and the rewards by account
    #[ink(message, payable, selector = 0xc218e5ba)]
    //#[openbrush::modifiers(access_control::only_role(REWARD_MANAGER_ROLE))]
    fn fund_rewards_and_add_winners(
        &mut self,
        era: u32,
        accounts: Vec<(AccountId, Balance)>,
    ) -> Result<(), RewardError> ;

    /// return true if the current account has pending rewards
    #[ink(message)]
    fn has_pending_rewards(&self) -> bool ;

    /// return true if the given account has pending rewards
    #[ink(message)]
    fn has_pending_rewards_from(&mut self, from: AccountId) -> bool ;

    /// return the pending rewards for a given account.
    #[ink(message)]
    //#[openbrush::modifiers(access_control::only_role(REWARD_VIEWER_ROLE))]
    fn get_pending_rewards_from(
        &mut self,
        from: AccountId,
    ) -> Result<Option<Balance>, RewardError> ;

    /// claim all pending rewards for the current account
    /// After claiming, there is not anymore pending rewards for this account
    #[ink(message)]
    fn claim(&mut self) -> Result<(), RewardError>;

    /// claim all pending rewards for the given account
    /// After claiming, there is not anymore pending rewards for this account
    #[ink(message)]
    fn claim_from(&mut self, from: AccountId) -> Result<(), RewardError> ;

}


pub trait Psp22RewardStorage {
    fn get_storage(&self) -> &Psp22RewardData;
    fn get_mut_storage(&mut self) -> &mut Psp22RewardData;
}


pub trait BasePsp22Reward: Psp22RewardStorage + BaseAccessControl {

    /// Add the accounts in the list of winners for a given era
    /// accounts contains the list of winners and the rewards by account
    fn inner_fund_rewards_and_add_winners(
        &mut self,
        era: u32,
        accounts: Vec<(AccountId, Balance)>,
    ) -> Result<(), RewardError> {

        let caller = ::ink::env::caller::<DefaultEnvironment>();
        self.inner_check_role(REWARD_MANAGER_ROLE, caller)?;

        let transferred_value = ::ink::env::transferred_value::<DefaultEnvironment>();
        let mut total_rewards = Balance::default();

        // iterate on the accounts (the winners)
        for (account, reward) in accounts {
            total_rewards = total_rewards.checked_add(reward).ok_or(RewardError::AddOverFlow)?;

            // compute the new rewards for this winner
            let new_reward = match Psp22RewardStorage::get_storage(self).pending_rewards.get(account) {
                Some(existing_reward) => existing_reward.checked_add(reward).ok_or(RewardError::AddOverFlow)?,
                _ => reward,
            };

            // add the pending rewards for this account
            Psp22RewardStorage::get_mut_storage(self)
                .pending_rewards
                .insert(account, &new_reward);

            // emit the event
            ::ink::env::emit_event::<DefaultEnvironment, PendingReward>(
                PendingReward{account, era, amount:reward}
            );
        }

        if transferred_value < total_rewards {
            return Err(RewardError::InsufficientTransferredBalance);
        }

        Ok(())
    }

    /*
    /// return true if the current account has pending rewards
    fn inner_has_pending_rewards(&self) -> bool {
        let from = Self::env().caller();
        self._has_pending_rewards_from(from)
    }
     */

    fn inner_has_pending_rewards_from(&self, from: AccountId) -> bool {
        Psp22RewardStorage::get_storage(self).pending_rewards.contains(from)
    }

    /// return the pending rewards for a given account.
    fn inner_get_pending_rewards_from(
        &mut self,
        from: AccountId,
    ) -> Result<Option<Balance>, RewardError> {
        Ok(Psp22RewardStorage::get_storage(self).pending_rewards.get(from))
    }

    /// claim all pending rewards for the current account
    /// After claiming, there is not anymore pending rewards for this account
    ///
    /*
    fn claim(&mut self) -> Result<(), RewardError> {
        let from = Self::env().caller();
        self._claim_from(from)
    }
        /// claim all pending rewards for the given account
    /// After claiming, there is not anymore pending rewards for this account
    #[ink(message)]
    fn claim_from(&mut self, from: AccountId) -> Result<(), RewardError> {
        self._claim_from(from)
    }

     */


    fn inner_claim_from(&mut self, from: AccountId) -> Result<(), RewardError> {
        // get all pending rewards for this account
        match Psp22RewardStorage::get_storage(self).pending_rewards.get(&from) {
            Some(pending_rewards) => {
                // remove the pending rewards
                Psp22RewardStorage::get_mut_storage(self).pending_rewards.remove(&from);
                // emit the event
                ::ink::env::emit_event::<DefaultEnvironment, RewardsClaimed>(
                    RewardsClaimed{account:from, amount:pending_rewards}
                );


                // transfer the amount
                ::ink::env::transfer::<DefaultEnvironment>(from, pending_rewards)
                    .map_err(|_| RewardError::TransferError)?;
                Ok(())
            }
            _ => Err(RewardError::NoReward),
        }
    }
}
