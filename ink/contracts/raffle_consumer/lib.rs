#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod raffle_consumer {
    use ink::env::call::{ExecutionInput, Selector};
    use ink::env::debug_message;
    use ink::prelude::vec::Vec;
    use inkv5_client_lib::only_role;
    use inkv5_client_lib::traits::access_control::*;
    use inkv5_client_lib::traits::kv_store::*;
    use inkv5_client_lib::traits::message_queue::*;
    use inkv5_client_lib::traits::meta_transaction::*;
    use inkv5_client_lib::traits::rollup_client::*;
    use inkv5_client_lib::traits::*;
    use lucky::traits::error::RaffleError;

    use lucky::traits::{participant_filter::filter_latest_winners, participant_filter::filter_latest_winners::*, raffle, raffle::*, RAFFLE_MANAGER_ROLE};

    // Selector of withdraw: "0x410fcc9d"
    const WITHDRAW_SELECTOR: [u8; 4] = [0x41, 0x0f, 0xcc, 0x9d];
    // Selector of Psp22Reward::fund_rewards_and_add_winners": "0xc218e5ba"
    const FUND_REWARDS_AND_WINNERS_SELECTOR: [u8; 4] = [0xc2, 0x18, 0xe5, 0xba];

    /// Event emitted when the Raffle is done
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

    /// Contract storage
    #[derive(Default)]
    #[ink(storage)]
    pub struct Contract {
        access_control: AccessControlData,
        kv_store: KvStoreData,
        meta_transaction: MetaTransactionData,
        /// data linked to the dApps
        dapps_staking_developer_address: Option<AccountId>,
        reward_manager_address: Option<AccountId>,
        raffle: raffle::RaffleData,
        filter_latest_winners: filter_latest_winners::FilterLatestWinnersData,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new(
            dapps_staking_developer_address: AccountId,
            reward_manager_address: AccountId,
        ) -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            // set the admin of this contract
            BaseAccessControl::init_with_admin(&mut instance, caller);
            BaseAccessControl::inner_grant_role(&mut instance, RAFFLE_MANAGER_ROLE, caller)
                .expect("Should grant the role RAFFLE_MANAGER_ROLE");
            instance.dapps_staking_developer_address = Some(dapps_staking_developer_address);
            instance.reward_manager_address = Some(reward_manager_address);
            instance
        }

        pub fn save_response(
            &mut self,
            response: &RaffleResponseMessage,
        ) -> Result<(), RaffleError> {
            if response.skipped {
                self.skip_raffle(response.era)?;
                // emit event RaffleSkipped
                self.env().emit_event(RaffleSkipped {
                    contract: self.env().caller(),
                    era: response.era,
                });

                return Ok(());
            }

            let winners_rewards =
                self.mark_raffle_done(response.era, response.rewards, &response.winners)?;

            let nb_winners = winners_rewards.len();

            // save the winners
            let mut given_rewards : Balance = 0;
            for winner in &winners_rewards {
                self.add_winner(winner.0);
                given_rewards = given_rewards.checked_add(winner.1).ok_or(RaffleError::AddOverFlow)? ;
            }

            // withdraw the rewards from developer dAppsStaking
            let dapps_staking_developer_address = self
                .dapps_staking_developer_address
                .ok_or(RaffleError::DappsStakingDeveloperAddressMissing)?;

            debug_message("call dAppStaking dev contract");
            ink::env::call::build_call::<Environment>()
                .call(dapps_staking_developer_address)
                .call_v1()
                .exec_input(
                    ExecutionInput::new(Selector::new(WITHDRAW_SELECTOR)).push_arg(given_rewards),
                )
                .returns::<Result<(), RaffleError>>()
                .invoke()
                .or(Err(RaffleError::CrossContractCallError1))?;

            // set the list of winners and fund the rewards
            let reward_manager_address = self
                .reward_manager_address
                .ok_or(RaffleError::RewardManagerAddressMissing)?;

            debug_message("call reward manager contract");
            ink::env::call::build_call::<Environment>()
                .call(reward_manager_address)
                .call_v1()
                .transferred_value(given_rewards)
                .exec_input(
                    ExecutionInput::new(Selector::new(FUND_REWARDS_AND_WINNERS_SELECTOR))
                        .push_arg(response.era)
                        .push_arg(winners_rewards),
                )
                .returns::<Result<(), RaffleError>>()
                .invoke()
                .or(Err(RaffleError::CrossContractCallError2))?;

            // emit event RaffleDone
            self.env().emit_event(RaffleDone {
                contract: self.env().caller(),
                era: response.era,
                nb_winners: u16::try_from(nb_winners)?,
                pending_rewards: response.rewards,
            });

            Ok(())
        }

        #[ink(message)]
        pub fn set_dapps_staking_developer_address(
            &mut self,
            address: AccountId,
        ) -> Result<(), RaffleError> {
            only_role!(self, ADMIN_ROLE);
            self.dapps_staking_developer_address = Some(address);
            Ok(())
        }

        #[ink(message)]
        pub fn get_dapps_staking_developer_address(&mut self) -> Option<AccountId> {
            self.dapps_staking_developer_address
        }

        #[ink(message)]
        pub fn set_reward_manager_address(
            &mut self,
            address: AccountId,
        ) -> Result<(), RaffleError> {
            only_role!(self, ADMIN_ROLE);
            self.reward_manager_address = Some(address);
            Ok(())
        }

        #[ink(message)]
        pub fn get_reward_manager_address(&mut self) -> Option<AccountId> {
            self.reward_manager_address
        }

        #[ink(message)]
        pub fn register_attestor(
            &mut self,
            account_id: AccountId,
        ) -> Result<(), AccessControlError> {
            only_role!(self, ADMIN_ROLE);
            AccessControl::grant_role(self, ATTESTOR_ROLE, account_id)?;
            Ok(())
        }

        #[ink(message)]
        pub fn get_attestor_role(&self) -> RoleType {
            ATTESTOR_ROLE
        }

        #[ink(message)]
        pub fn terminate_me(&mut self) -> Result<(), RaffleError> {
            only_role!(self, ADMIN_ROLE);
            self.env().terminate_contract(self.env().caller());
        }

        #[ink(message)]
        pub fn withdraw(&mut self, value: Balance) -> Result<(), RaffleError> {
            only_role!(self, ADMIN_ROLE);
            let caller = Self::env().caller();
            self.env()
                .transfer(caller, value)
                .map_err(|_| RaffleError::TransferError)?;
            Ok(())
        }

    }

    #[ink::scale_derive(Encode, Decode)]
    pub struct RaffleResponseMessage {
        pub era: u32,
        pub skipped: bool,
        pub rewards: Balance,
        pub winners: Vec<AccountId>,
    }

    /// Implement the business logic for the Rollup Client in the 'on_message_received' method
    impl BaseRollupClient for Contract {
        fn on_message_received(&mut self, action: Vec<u8>) -> Result<(), RollupClientError> {

            // parse the response
            let response: RaffleResponseMessage = ink::scale::Decode::decode(&mut &action[..])
                .or(Err(RollupClientError::FailedToDecode))?;

            self.save_response(&response)?;

            Ok(())
        }
    }

    /// Boilerplate code to manage the Raffle
    impl RaffleStorage for Contract {
        fn get_storage(&self) -> &RaffleData {
            &self.raffle
        }

        fn get_mut_storage(&mut self) -> &mut RaffleData {
            &mut self.raffle
        }
    }

    impl BaseRaffle for Contract {}

    impl Raffle for Contract {

        #[ink(message)]
        fn set_ratio_distribution(
            &mut self,
            ratio: Vec<Balance>,
            total_ratio: Balance,
        ) -> Result<(), RaffleError> {
            self.inner_set_ratio_distribution(ratio, total_ratio)
        }

        #[ink(message)]
        fn get_ratio_distribution(&self) -> Vec<Balance> {
            self.inner_get_ratio_distribution()
        }

        #[ink(message)]
        fn get_total_ratio_distribution(&self) -> Balance {
            self.inner_get_total_ratio_distribution()
        }

        #[ink(message)]
        fn get_next_era(&self) -> Result<u32, RaffleError> {
            self.inner_get_next_era()
        }

        #[ink(message)]
        fn set_next_era(&mut self, next_era: u32) -> Result<(), RaffleError> {
            self.inner_set_next_era(next_era)
        }
    }

    /// Boilerplate code to manage the FilterLatestWinners
    impl FilterLatestWinnersStorage for Contract {
        fn get_storage(&self) -> &FilterLatestWinnersData {
            &self.filter_latest_winners
        }

        fn get_mut_storage(&mut self) -> &mut FilterLatestWinnersData {
            &mut self.filter_latest_winners
        }
    }

    impl BaseFilterLatestWinners for Contract {}

    impl FilterLatestWinners for Contract {
        #[ink(message)]
        fn set_nb_winners_filtered(
            &mut self,
            nb_filtered_winners: u16,
        ) -> Result<(), RaffleError> {
            self.inner_set_nb_winners_filtered(nb_filtered_winners)
        }

        #[ink(message)]
        fn get_nb_winners_filtered(&self) -> u16 {
            self.inner_get_nb_winners_filtered()
        }

        #[ink(message)]
        fn get_last_winners(&self) -> Vec<AccountId> {
            self.inner_get_last_winners()
        }

        #[ink(message)]
        fn add_address_in_last_winner(
            &mut self,
            winner: AccountId,
        ) -> Result<(), RaffleError> {
            self.inner_add_address_in_last_winner(winner)
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

    /// Boilerplate code to implement the Key Value Store
    impl KvStoreStorage for Contract {
        fn get_storage(&self) -> &KvStoreData {
            &self.kv_store
        }

        fn get_mut_storage(&mut self) -> &mut KvStoreData {
            &mut self.kv_store
        }
    }

    impl KvStore for Contract {}

    /// Boilerplate code to implement the Message Queue
    impl MessageQueue for Contract {}

    /// Boilerplate code to implement the Rollup Client
    impl RollupClient for Contract {
        #[ink(message)]
        fn get_value(&self, key: Key) -> Option<Value> {
            self.inner_get_value(&key)
        }

        #[ink(message)]
        fn has_message(&self) -> Result<bool, RollupClientError> {
            MessageQueue::has_message(self)
        }

        #[ink(message)]
        fn rollup_cond_eq(
            &mut self,
            conditions: Vec<(Key, Option<Value>)>,
            updates: Vec<(Key, Option<Value>)>,
            actions: Vec<HandleActionInput>,
        ) -> Result<(), RollupClientError> {
            self.inner_rollup_cond_eq(conditions, updates, actions)
        }
    }

    /// Boilerplate code to implement the Meta Transaction
    impl MetaTransactionStorage for Contract {
        fn get_storage(&self) -> &MetaTransactionData {
            &self.meta_transaction
        }

        fn get_mut_storage(&mut self) -> &mut MetaTransactionData {
            &mut self.meta_transaction
        }
    }

    impl BaseMetaTransaction for Contract {}

    impl MetaTransaction for Contract {
        #[ink(message)]
        fn prepare(
            &self,
            from: AccountId,
            data: Vec<u8>,
        ) -> Result<(ForwardRequest, Hash), RollupClientError> {
            self.inner_prepare(from, data)
        }

        #[ink(message)]
        fn meta_tx_rollup_cond_eq(
            &mut self,
            request: ForwardRequest,
            signature: [u8; 65],
        ) -> Result<(), RollupClientError> {
            self.inner_meta_tx_rollup_cond_eq(request, signature)
        }
    }

}
