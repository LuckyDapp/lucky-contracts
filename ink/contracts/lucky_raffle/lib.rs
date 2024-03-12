#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[openbrush::implementation(Ownable, AccessControl, Upgradeable)]
#[openbrush::contract]
pub mod raffle_contract {
    use ink::codegen::{EmitEvent, Env};
    use ink::env::call::{ExecutionInput, Selector};
    use ink::prelude::vec::Vec;
    use openbrush::contracts::access_control::*;
    use openbrush::contracts::ownable::*;
    use openbrush::{modifiers, traits::Storage};

    use lucky::traits::{
        RAFFLE_MANAGER_ROLE,
        participant_filter::filter_latest_winners,
        participant_filter::filter_latest_winners::*,
        raffle, raffle::*,
    };

    use phat_rollup_anchor_ink::traits::{
        meta_transaction, meta_transaction::*,
        rollup_anchor, rollup_anchor::*,
        js_rollup_anchor, js_rollup_anchor::*,
    };

    // Selector of withdraw: "0x410fcc9d"
    const WITHDRAW_SELECTOR: [u8; 4] = [0x41, 0x0f, 0xcc, 0x9d];
    // Selector of Psp22Reward::fund_rewards_and_add_winners": ""0xc218e5ba
    const FUND_REWARDS_AND_WINNERS_SELECTOR: [u8; 4] = [0xc2, 0x18, 0xe5, 0xba];

    /// Event emitted when the Rafle is done
    #[ink(event)]
    pub struct RaffleDone {
        #[ink(topic)]
        contract: AccountId,
        #[ink(topic)]
        era: u32,
        pending_rewards: Balance,
        nb_winners: u16,
    }

    #[ink(event)]
    pub struct RaffleSkipped {
        #[ink(topic)]
        contract: AccountId,
        #[ink(topic)]
        era: u32,
    }

    #[ink(event)]
    pub struct ErrorReceived {
        /// era requested
        era: u32,
        /// error
        error: Vec<u8>,
        /// when the error has been received
        timestamp: u64,
    }


    /// Errors occurred in the contract
    #[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractError {
        AccessControlError(AccessControlError),
        RaffleError(RaffleError),
        CrossContractCallError1,
        CrossContractCallError2,
        TransferError,
        DappsStakingDeveloperAddressMissing,
        RewardManagerAddressMissing,
    }

    /// convertor from AccessControlError to ContractError
    impl From<AccessControlError> for ContractError {
        fn from(error: AccessControlError) -> Self {
            ContractError::AccessControlError(error)
        }
    }

    /// convertor from RaffleError to ContractError
    impl From<RaffleError> for ContractError {
        fn from(error: RaffleError) -> Self {
            ContractError::RaffleError(error)
        }
    }

    /// convertor from RaffleError to ContractError
    impl From<ContractError> for RollupAnchorError {
        fn from(error: ContractError) -> Self {
            ink::env::debug_println!("Error: {:?}", error);
            RollupAnchorError::UnsupportedAction
        }
    }

    /// Contract storage
    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        access: access_control::Data,
        #[storage_field]
        rollup_anchor: rollup_anchor::Data,
        #[storage_field]
        meta_transaction: meta_transaction::Data,
        #[storage_field]
        js_rollup_anchor: js_rollup_anchor::Data,
        /// data linked to the dApps
        dapps_staking_developer_address: Option<AccountId>,
        reward_manager_address: Option<AccountId>,
        #[storage_field]
        raffle: raffle::Data,
        #[storage_field]
        filter_latest_winners: filter_latest_winners::Data,
    }

    impl Raffle for Contract {}
    impl FilterLatestWinners for Contract {}
    impl RollupAnchor for Contract {}
    impl MetaTransaction for Contract {}
    impl JsRollupAnchor for Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new(
            dapps_staking_developer_address: AccountId,
            reward_manager_address: AccountId,
        ) -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            // set the owner of this contract
            ownable::Internal::_init_with_owner(&mut instance, caller);
            // set the admin of this contract
            access_control::Internal::_init_with_admin(&mut instance, Some(caller));
            // grant the role manager
            AccessControl::grant_role(&mut instance, JS_RA_MANAGER_ROLE, Some(caller))
                .expect("Should grant the role JS_RA_MANAGER_ROLE");
            AccessControl::grant_role(&mut instance, RAFFLE_MANAGER_ROLE, Some(caller))
                .expect("Should grant the role RAFFLE_MANAGER_ROLE");
            instance.dapps_staking_developer_address = Some(dapps_staking_developer_address);
            instance.reward_manager_address = Some(reward_manager_address);
            instance
        }

        pub fn save_response(
            &mut self,
            response: &RaffleResponseMessage,
        ) -> Result<(), ContractError> {

            if response.skipped {
                self.skip_raffle(response.era)?;
                // emit event RaffleSkipped
                self.env().emit_event(RaffleSkipped {
                    contract: self.env().caller(),
                    era: response.era,
                });

                return Ok(());
            }

            let winners_rewards = self.mark_raffle_done(response.era, response.rewards, &response.winners)?;

            let nb_winners = winners_rewards.len();

            // save the winners
            let mut given_rewards = 0;
            for winner in &winners_rewards {
                self.add_winner(winner.0);
                given_rewards += winner.1;
            }

            // withdraw the rewards from developer dAppsStaking
            let dapps_staking_developer_address = self
                .dapps_staking_developer_address
                .ok_or(ContractError::DappsStakingDeveloperAddressMissing)?;
            ink::env::call::build_call::<Environment>()
                .call(dapps_staking_developer_address)
                .exec_input(ExecutionInput::new(Selector::new(WITHDRAW_SELECTOR)).push_arg(given_rewards))
                .returns::<()>()
                .invoke();

            // set the list of winners and fund the rewards
            let reward_manager_address = self
                .reward_manager_address
                .ok_or(ContractError::RewardManagerAddressMissing)?;
            ink::env::call::build_call::<Environment>()
                .call(reward_manager_address)
                .transferred_value(given_rewards)
                .exec_input(
                    ExecutionInput::new(Selector::new(FUND_REWARDS_AND_WINNERS_SELECTOR))
                        .push_arg(response.era)
                        .push_arg(winners_rewards),
                )
                .returns::<()>()
                .invoke();

            // emit event RaffleDone
            self.env().emit_event(RaffleDone {
                contract: self.env().caller(),
                era: response.era,
                nb_winners: nb_winners as u16,
                pending_rewards: response.rewards,
            });

            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn set_dapps_staking_developer_address(
            &mut self,
            address: AccountId,
        ) -> Result<(), ContractError> {
            self.dapps_staking_developer_address = Some(address);
            Ok(())
        }

        #[ink(message)]
        pub fn get_dapps_staking_developer_address(&mut self) -> Option<AccountId> {
            self.dapps_staking_developer_address
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn set_reward_manager_address(
            &mut self,
            address: AccountId,
        ) -> Result<(), ContractError> {
            self.reward_manager_address = Some(address);
            Ok(())
        }

        #[ink(message)]
        pub fn get_reward_manager_address(&mut self) -> Option<AccountId> {
            self.reward_manager_address
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn terminate_me(&mut self) -> Result<(), ContractError> {
            self.env().terminate_contract(self.env().caller());
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn withdraw(&mut self, value: Balance) -> Result<(), ContractError> {
            let caller = Self::env().caller();
            self.env()
                .transfer(caller, value)
                .map_err(|_| ContractError::TransferError)?;
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn register_attestor(&mut self, account_id: AccountId) -> Result<(), AccessControlError> {
            AccessControl::grant_role(self, ATTESTOR_ROLE, Some(account_id))?;
            Ok(())
        }

        #[ink(message)]
        pub fn get_attestor_role(&self) -> RoleType {
            ATTESTOR_ROLE
        }

    }

    #[derive(scale::Encode, scale::Decode)]
    pub struct RaffleRequestMessage {
        pub era: u32,
        pub nb_winners: u16,
        pub excluded: Vec<AccountId>,
    }

    #[derive(scale::Encode, scale::Decode)]
    pub struct RaffleResponseMessage {
        pub era: u32,
        pub skipped: bool,
        pub rewards: Balance,
        pub winners: Vec<AccountId>,
    }

    impl rollup_anchor::MessageHandler for Contract {
        fn on_message_received(&mut self, action: Vec<u8>) -> Result<(), RollupAnchorError> {

            let response = JsRollupAnchor::on_message_received::<RaffleRequestMessage, RaffleResponseMessage>(self, action)?;
            match response {
                MessageReceived::Ok {output} => {
                    // register the info
                    self.save_response(&output)?;

                }
                MessageReceived::Error { error, input } => {
                    // we received an error
                    let timestamp = self.env().block_timestamp();
                    self.env().emit_event(ErrorReceived {
                        era : input.era,
                        error,
                        timestamp,
                    });
                }
            }

            Ok(())
        }
    }

    impl rollup_anchor::EventBroadcaster for Contract {
        fn emit_event_message_queued(&self, _id: u32, _data: Vec<u8>) {
            // no queue here
        }
        fn emit_event_message_processed_to(&self, _id: u32) {
            // no queue here
        }
    }

    impl meta_transaction::EventBroadcaster for Contract {
        fn emit_event_meta_tx_decoded(&self) {
            // do nothing
        }
    }

}
