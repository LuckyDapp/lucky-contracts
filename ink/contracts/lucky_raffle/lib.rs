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
        fn from(_error: ContractError) -> Self {
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
                .expect("Should grant the role MANAGER_ROLE");
            instance.dapps_staking_developer_address = Some(dapps_staking_developer_address);
            instance.reward_manager_address = Some(reward_manager_address);
            instance
        }

        pub fn save_raffle_result(
            &mut self,
            era: u32,
            rewards: Balance,
            winners: Vec<AccountId>,
        ) -> Result<(), ContractError> {

            let winners_rewards = self.mark_raffle_done(era, rewards, &winners)?;

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
                        .push_arg(era)
                        .push_arg(winners_rewards),
                )
                .returns::<()>()
                .invoke();

            // emit event RaffleDone
            self.env().emit_event(RaffleDone {
                contract: self.env().caller(),
                era,
                nb_winners: nb_winners as u16,
                pending_rewards: rewards,
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
        era: u32,
        nb_winners: u32,
        excluded: Vec<AccountId>,
    }

    #[derive(scale::Encode, scale::Decode)]
    pub struct RaffleResponseMessage {
        era: u32,
        rewards: Balance,
        winners: Vec<AccountId>,
    }

    impl rollup_anchor::MessageHandler for Contract {
        fn on_message_received(&mut self, action: Vec<u8>) -> Result<(), RollupAnchorError> {

            let response = JsRollupAnchor::on_message_received::<RaffleRequestMessage, RaffleResponseMessage>(self, action)?;
            match response {
                MessageReceived::Ok {output} => {

                    let era = output.era;
                    let rewards = output.rewards;
                    let winners = output.winners;

                    // register the info
                    self.save_raffle_result(era, rewards, winners)?;

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

    
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use openbrush::contracts::access_control::accesscontrol_external::AccessControl;

        use ink::env::DefaultEnvironment;
        use ink_e2e::subxt::tx::Signer;
        use ink_e2e::{build_message, PolkadotConfig};
        use scale::Encode;

        use phat_rollup_anchor_ink::traits::{
            meta_transaction::metatransaction_external::MetaTransaction,
            rollup_anchor::rollupanchor_external::RollupAnchor,
            js_rollup_anchor::jsrollupanchor_external::JsRollupAnchor,
            js_rollup_anchor::ResponseMessage::{Error, JsResponse},
        };

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        async fn alice_instantiates_contract(
            client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
        ) -> AccountId {
            let constructor = ContractRef::new();
            client
                .instantiate(
                    "raffle_contract",
                    &ink_e2e::alice(),
                    constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id
        }

        async fn alice_set_js_script_hash(
            client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
            contract_id: &AccountId,
        ) {
            let code_hash = [1u8; 32];
            let set_js_script_hash = build_message::<ContractRef>(contract_id.clone())
                .call(|oracle| oracle.set_js_script_hash(code_hash));
            client
                .call(&ink_e2e::alice(), set_js_script_hash, 0, None)
                .await
                .expect("set js code hash failed");
        }

        async fn alice_set_settings_hash(
            client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
            contract_id: &AccountId,
        ) {
            let code_hash = [2u8; 32];
            let set_settings_hash = build_message::<ContractRef>(contract_id.clone())
                .call(|oracle| oracle.set_settings_hash(code_hash));
            client
                .call(&ink_e2e::alice(), set_settings_hash, 0, None)
                .await
                .expect("set settings hash failed");
        }

        async fn alice_grants_bob_as_attestor(
            client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
            contract_id: &AccountId,
        ) {
            // bob is granted as attestor
            let bob_address = ink::primitives::AccountId::from(ink_e2e::bob().public_key().0);
            let grant_role = build_message::<ContractRef>(contract_id.clone())
                .call(|oracle| oracle.grant_role(ATTESTOR_ROLE, Some(bob_address)));
            client
                .call(&ink_e2e::alice(), grant_role, 0, None)
                .await
                .expect("grant bob as attestor failed");
        }

        #[ink_e2e::test]
        async fn test_receive_data(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let contract_id = alice_instantiates_contract(&mut client).await;

            // set the js code hash
            alice_set_js_script_hash(&mut client, &contract_id).await;

            // set the settings code hash
            alice_set_settings_hash(&mut client, &contract_id).await;

            // bob is granted as attestor
            alice_grants_bob_as_attestor(&mut client, &contract_id).await;

            let dave_address = ink::primitives::AccountId::from(ink_e2e::dave().public_key().0);

            // data is received
            let response = RaffleResponseMessage {
                era: 12,
                rewards: 9958,
                winners: [dave_address].to_vec(),
            };

            let payload = JsResponse {
                js_script_hash: [1u8; 32],
                input_hash: [3u8; 32],
                settings_hash: [2u8; 32],
                output_value: response.encode(),
            };
            let actions = vec![HandleActionInput::Reply(payload.encode())];
            let rollup_cond_eq = build_message::<ContractRef>(contract_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], actions.clone()));
            let result = client
                .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
                .await
                .expect("rollup cond eq should be ok");
            // two events : MessageProcessedTo and ValueReceived
            assert!(result.contains_event("Contracts", "ContractEmitted"));

            // and check if the data is filled
            let get_last_era_done = build_message::<ContractRef>(contract_id.clone())
                .call(|oracle| oracle.get_last_era_done());
            let get_res = client
                .call_dry_run(&ink_e2e::charlie(), &get_last_era_done, 0, None)
                .await;
            let last_era_done = get_res.return_value().expect("Last era not found");

            assert_eq!(12, last_era_done);

            Ok(())
        }

        #[ink_e2e::test]
        async fn test_receive_error(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let contract_id = alice_instantiates_contract(&mut client).await;

            // set the js code hash
            alice_set_js_script_hash(&mut client, &contract_id).await;

            // set the settings code hash
            alice_set_settings_hash(&mut client, &contract_id).await;

            // bob is granted as attestor
            alice_grants_bob_as_attestor(&mut client, &contract_id).await;

            let input_data = RaffleRequestMessage {
                era: 101,
                nb_winners: 1,
                excluded: [].to_vec(),
            };

            // then a response is received
            let error = vec![3u8; 5];
            let payload = Error {
                js_script_hash: [1u8; 32],
                input_value: input_data.encode(),
                settings_hash: [2u8; 32],
                error,
            };
            let actions = vec![HandleActionInput::Reply(payload.encode())];
            let rollup_cond_eq = build_message::<ContractRef>(contract_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], actions.clone()));
            let result = client
                .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
                .await
                .expect("we should proceed error message");
            // two events : MessageProcessedTo and ErrorReceived
            assert!(result.contains_event("Contracts", "ContractEmitted"));

            Ok(())
        }

        #[ink_e2e::test]
        async fn test_bad_attestor(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let contract_id = alice_instantiates_contract(&mut client).await;

            // set the js code hash
            alice_set_js_script_hash(&mut client, &contract_id).await;

            // set the settings code hash
            alice_set_settings_hash(&mut client, &contract_id).await;

            // bob is not granted as attestor => it should not be able to send a message
            let rollup_cond_eq = build_message::<ContractRef>(contract_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], vec![]));
            let result = client.call(&ink_e2e::bob(), rollup_cond_eq, 0, None).await;
            assert!(
                result.is_err(),
                "only attestor should be able to send messages"
            );

            // bob is granted as attestor
            alice_grants_bob_as_attestor(&mut client, &contract_id).await;

            // then bob is able to send a message
            let rollup_cond_eq = build_message::<ContractRef>(contract_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], vec![]));
            let result = client
                .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
                .await
                .expect("rollup cond eq failed");
            // no event
            assert!(!result.contains_event("Contracts", "ContractEmitted"));

            Ok(())
        }

        #[ink_e2e::test]
        async fn test_bad_hash(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let contract_id = alice_instantiates_contract(&mut client).await;

            // set the js code hash
            alice_set_js_script_hash(&mut client, &contract_id).await;

            // set the settings code hash
            alice_set_settings_hash(&mut client, &contract_id).await;

            // bob is granted as attestor
            alice_grants_bob_as_attestor(&mut client, &contract_id).await;

            // a response is received
            let dave_address = ink::primitives::AccountId::from(ink_e2e::dave().public_key().0);
            let response = RaffleResponseMessage {
                era: 12,
                rewards: 9958,
                winners: [dave_address].to_vec(),
            };
            let payload = JsResponse {
                js_script_hash: [9u8; 32],
                input_hash: [3u8; 32],
                settings_hash: [2u8; 32],
                output_value: response.encode(),
            };
            let actions = vec![HandleActionInput::Reply(payload.encode())];
            let rollup_cond_eq = build_message::<ContractRef>(contract_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], actions.clone()));
            let result = client.call(&ink_e2e::bob(), rollup_cond_eq, 0, None).await;
            assert!(
                result.is_err(),
                "We should not accept response with bad js code hash"
            );

            let payload = JsResponse {
                js_script_hash: [1u8; 32],
                input_hash: [3u8; 32],
                settings_hash: [9u8; 32],
                output_value: response.encode(),
            };
            let actions = vec![HandleActionInput::Reply(payload.encode())];
            let rollup_cond_eq = build_message::<ContractRef>(contract_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], actions.clone()));
            let result = client.call(&ink_e2e::bob(), rollup_cond_eq, 0, None).await;
            assert!(
                result.is_err(),
                "We should not accept response with bad settings code hash"
            );

            Ok(())
        }

        #[ink_e2e::test]
        async fn test_bad_messages(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given

            let contract_id = alice_instantiates_contract(&mut client).await;

            // set the js code hash
            alice_set_js_script_hash(&mut client, &contract_id).await;

            // set the settings code hash
            alice_set_settings_hash(&mut client, &contract_id).await;

            // bob is granted as attestor
            alice_grants_bob_as_attestor(&mut client, &contract_id).await;

            let actions = vec![HandleActionInput::Reply(58u128.encode())];
            let rollup_cond_eq = build_message::<ContractRef>(contract_id.clone())
                .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], actions.clone()));
            let result = client.call(&ink_e2e::bob(), rollup_cond_eq, 0, None).await;
            assert!(
                result.is_err(),
                "we should not be able to proceed bad messages"
            );

            Ok(())
        }

        ///
        /// Test the meta transactions
        /// Alice is the owner
        /// Bob is the attestor
        /// Charlie is the sender (ie the payer)
        ///
        #[ink_e2e::test]
        async fn test_meta_tx_rollup_cond_eq(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let contract_id = alice_instantiates_contract(&mut client).await;

            // Bob is the attestor
            // use the ecsda account because we are not able to verify the sr25519 signature
            let from = ink::primitives::AccountId::from(
                Signer::<PolkadotConfig>::account_id(&subxt_signer::ecdsa::dev::bob()).0,
            );

            // add the role => it should be succeed
            let grant_role = build_message::<ContractRef>(contract_id.clone())
                .call(|oracle| oracle.grant_role(ATTESTOR_ROLE, Some(from)));
            client
                .call(&ink_e2e::alice(), grant_role, 0, None)
                .await
                .expect("grant the attestor failed");

            // prepare the meta transaction
            let data = RollupCondEqMethodParams::encode(&(vec![], vec![], vec![]));
            let prepare_meta_tx = build_message::<ContractRef>(contract_id.clone())
                .call(|oracle| oracle.prepare(from, data.clone()));
            let result = client
                .call(&ink_e2e::bob(), prepare_meta_tx, 0, None)
                .await
                .expect("We should be able to prepare the meta tx");

            let (request, _hash) = result
                .return_value()
                .expect("Expected value when preparing meta tx");

            assert_eq!(0, request.nonce);
            assert_eq!(from, request.from);
            assert_eq!(contract_id, request.to);
            assert_eq!(&data, &request.data);

            // Bob signs the message
            let keypair = subxt_signer::ecdsa::dev::bob();
            let signature = keypair.sign(&scale::Encode::encode(&request)).0;

            // do the meta tx: charlie sends the message
            let meta_tx_rollup_cond_eq =
                build_message::<ContractRef>(contract_id.clone())
                    .call(|oracle| oracle.meta_tx_rollup_cond_eq(request.clone(), signature));
            client
                .call(&ink_e2e::charlie(), meta_tx_rollup_cond_eq, 0, None)
                .await
                .expect("meta tx rollup cond eq should not failed");

            // do it again => it must failed
            let meta_tx_rollup_cond_eq =
                build_message::<ContractRef>(contract_id.clone())
                    .call(|oracle| oracle.meta_tx_rollup_cond_eq(request.clone(), signature));
            let result = client
                .call(&ink_e2e::charlie(), meta_tx_rollup_cond_eq, 0, None)
                .await;
            assert!(
                result.is_err(),
                "This message should not be proceed because the nonce is obsolete"
            );

            Ok(())
        }
    }

}
