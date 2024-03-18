#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[openbrush::implementation(AccessControl)]
#[openbrush::contract]
pub mod reward_manager {
    use ink::codegen::{EmitEvent, Env};
    use lucky::traits::{reward::psp22_reward, reward::psp22_reward::*};
    use openbrush::contracts::access_control::{AccessControl, AccessControlError, RoleType, *};
    use openbrush::{modifiers, traits::Storage};

    const WHITELISTED_ADDRESS: RoleType = ink::selector_id!("WHITELISTED_ADDRESS");

    /// Event emitted when a reward is pending
    #[ink(event)]
    pub struct PendingReward {
        #[ink(topic)]
        account: AccountId,
        era: u32,
        amount: Balance,
    }

    /// Event emitted when a user claim rewards
    #[ink(event)]
    pub struct RewardsClaimed {
        #[ink(topic)]
        account: AccountId,
        amount: Balance,
    }

    /// Errors occurred in the contract
    #[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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
    impl From<access_control::AccessControlError> for ContractError {
        fn from(error: AccessControlError) -> Self {
            ContractError::AccessControlError(error)
        }
    }

    /// Contract storage
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        reward: psp22_reward::Data,
        #[storage_field]
        access: access_control::Data,
    }

    /// implementations of the contracts
    impl Psp22Reward for Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            access_control::Internal::_init_with_admin(&mut instance, Some(caller));
            AccessControl::grant_role(&mut instance, REWARD_MANAGER_ROLE, Some(caller))
                .expect("Should grant the role REWARD_MANAGER");
            AccessControl::grant_role(&mut instance, REWARD_VIEWER_ROLE, Some(caller))
                .expect("Should grant the role REWARD_VIEWER");
            instance
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn upgrade_contract(&mut self, new_code_hash: Hash) -> Result<(), ContractError> {
            self.env()
                .set_code_hash(&new_code_hash)
                .map_err(|_| ContractError::UpgradeError)?;
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn terminate_me(&mut self) -> Result<(), ContractError> {
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
        #[openbrush::modifiers(only_role(WHITELISTED_ADDRESS))]
        pub fn withdraw(&mut self, value: Balance) -> Result<(), ContractError> {
            let caller = Self::env().caller();
            self.env()
                .transfer(caller, value)
                .map_err(|_| ContractError::TransferError)?;
            Ok(())
        }
    }

    impl psp22_reward::Internal for Contract {
        fn _emit_pending_reward_event(&self, account: AccountId, era: u32, amount: Balance) {
            self.env().emit_event(PendingReward {
                account,
                era,
                amount,
            });
        }

        fn _emit_rewards_claimed_event(&self, account: AccountId, amount: Balance) {
            self.env().emit_event(RewardsClaimed { account, amount });
        }
    }
}
