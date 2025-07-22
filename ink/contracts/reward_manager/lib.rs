#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod reward_manager {

    use inkv5_client_lib::only_role;
    use inkv5_client_lib::traits::access_control::*;
    use lucky::traits::reward::{*, psp22_reward::*};
    use ink::prelude::vec::Vec;

    const WHITELISTED_ADDRESS: RoleType = ink::selector_id!("WHITELISTED_ADDRESS");

    /// Errors occurred in the contract
    #[derive(Debug, Eq, PartialEq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[allow(clippy::cast_possible_truncation)]
    pub enum ContractError {
        RewardError(RewardError),
        AccessControlError(AccessControlError),
        UpgradeError,
        TransferError,
    }

    /// convertor from RewardError to ContractError
    impl From<RewardError> for ContractError {
        fn from(error: RewardError) -> Self {
            ContractError::RewardError(error)
        }
    }

    /// convertor from AccessControlError to ContractError
    impl From<AccessControlError> for ContractError {
        fn from(error: AccessControlError) -> Self {
            ContractError::AccessControlError(error)
        }
    }

    /// Contract storage
    #[derive(Default)]
    #[ink(storage)]
    pub struct Contract {
        reward: Psp22RewardData,
        access_control: AccessControlData,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            BaseAccessControl::init_with_admin(&mut instance, caller);
            BaseAccessControl::inner_grant_role(&mut instance, REWARD_MANAGER_ROLE, caller)
                .expect("Should grant the role REWARD_MANAGER_ROLE");
            BaseAccessControl::inner_grant_role(&mut instance, REWARD_VIEWER_ROLE, caller)
                .expect("Should grant the role REWARD_VIEWER_ROLE");
            instance
        }

        #[ink(message)]
        pub fn upgrade_contract(&mut self, new_code_hash: Hash) -> Result<(), ContractError> {
            only_role!(self, ADMIN_ROLE);
            self.env()
                .set_code_hash(&new_code_hash)
                .map_err(|_| ContractError::UpgradeError)?;
            Ok(())
        }

        #[ink(message)]
        pub fn terminate_me(&mut self) -> Result<(), ContractError> {
            only_role!(self, ADMIN_ROLE);
            self.env().terminate_contract(self.env().caller());
        }

        #[ink(message)]
        pub fn get_role_reward_manager(&self) -> RoleType {
            REWARD_MANAGER_ROLE
        }

        #[ink(message)]
        pub fn get_role_reward_viewer(&self) -> RoleType {
            REWARD_VIEWER_ROLE
        }

        #[ink(message)]
        pub fn withdraw(&mut self, value: Balance) -> Result<(), ContractError> {
            only_role!(self, WHITELISTED_ADDRESS);
            let caller = Self::env().caller();
            self.env()
                .transfer(caller, value)
                .map_err(|_| ContractError::TransferError)?;
            Ok(())
        }
    }

    /// Boilerplate code to implement the psp22 reward
    impl Psp22RewardStorage for Contract {
        fn get_storage(&self) -> &Psp22RewardData {
            &self.reward
        }

        fn get_mut_storage(&mut self) -> &mut Psp22RewardData {
            &mut self.reward
        }
    }

    impl BasePsp22Reward for Contract {}

    impl Psp22Reward for Contract {

        /// Add the accounts in the list of winners for a given era
        /// accounts contains the list of winners and the rewards by account
        #[ink(message, payable, selector = 0xc218e5ba)]
        fn fund_rewards_and_add_winners(
            &mut self,
            era: u32,
            accounts: Vec<(AccountId, Balance)>,
        ) -> Result<(), RewardError> {
            self.inner_fund_rewards_and_add_winners(era, accounts)
        }

        /// return true if the current account has pending rewards
        #[ink(message)]
        fn has_pending_rewards(&self) -> bool {
            let from = Self::env().caller();
            self.inner_has_pending_rewards_from(from)
        }

        /// return true if the given account has pending rewards
        #[ink(message)]
        fn has_pending_rewards_from(&mut self, from: AccountId) -> bool {
            self.inner_has_pending_rewards_from(from)
        }

        /// return the pending rewards for a given account.
        #[ink(message)]
        fn get_pending_rewards_from(
            &mut self,
            from: AccountId,
        ) -> Result<Option<Balance>, RewardError> {
            only_role!(self, REWARD_VIEWER_ROLE);
            self.inner_get_pending_rewards_from(from)
        }

        /// claim all pending rewards for the current account
        /// After claiming, there is not anymore pending rewards for this account
        #[ink(message)]
        fn claim(&mut self) -> Result<(), RewardError> {
            let from = Self::env().caller();
            self.inner_claim_from(from)
        }

        /// claim all pending rewards for the given account
        /// After claiming, there is not anymore pending rewards for this account
        #[ink(message)]
        fn claim_from(&mut self, from: AccountId) -> Result<(), RewardError> {
            self.inner_claim_from(from)
        }

    }

    /// Boilerplate code to implement the access control
    impl AccessControlStorage for Contract {
        fn get_storage(&self) -> &AccessControlData {
            &self.access_control
        }

        fn get_mut_storage(&mut self) -> &mut AccessControlData {
            &mut self.access_control
        }
    }

    impl BaseAccessControl for Contract {}

    impl AccessControl for Contract {
        #[ink(message)]
        fn has_role(&self, role: RoleType, account: AccountId) -> bool {
            self.inner_has_role(role, account)
        }

        #[ink(message)]
        fn grant_role(
            &mut self,
            role: RoleType,
            account: AccountId,
        ) -> Result<(), AccessControlError> {
            self.inner_grant_role(role, account)
        }

        #[ink(message)]
        fn revoke_role(
            &mut self,
            role: RoleType,
            account: AccountId,
        ) -> Result<(), AccessControlError> {
            self.inner_revoke_role(role, account)
        }

        #[ink(message)]
        fn renounce_role(&mut self, role: RoleType) -> Result<(), AccessControlError> {
            self.inner_renounce_role(role)
        }
    }

}
