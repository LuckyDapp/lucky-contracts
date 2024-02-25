#![cfg_attr(not(feature = "std"), no_std, no_main)]
#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use openbrush::contracts::access_control::accesscontrol_external::AccessControl;

    use ink::env::DefaultEnvironment;
    use ink_e2e::subxt::tx::Signer;
    use ink_e2e::{build_message, PolkadotConfig};
    use scale::Encode;
    use openbrush::traits::AccountId;

    use lucky::traits::raffle::raffle_external::Raffle;
    use lucky_raffle::raffle_contract;
    use lucky_raffle::raffle_contract::RaffleResponseMessage;
    use lucky_raffle::raffle_contract::RaffleRequestMessage;
    use lucky::traits::reward::psp22_reward::REWARD_MANAGER_ROLE;

    use reward_manager::reward_manager;
    use dapps_staking_developer::dapps_staking_developer;
    use ::dapps_staking_developer::dapps_staking_developer::WHITELISTED_ADDRESS;

    use phat_rollup_anchor_ink::traits::meta_transaction::metatransaction_external::MetaTransaction;
    use phat_rollup_anchor_ink::traits::rollup_anchor::rollupanchor_external::RollupAnchor;
    use phat_rollup_anchor_ink::traits::js_rollup_anchor::jsrollupanchor_external::JsRollupAnchor;


    use phat_rollup_anchor_ink::traits::{
        rollup_anchor::*,
        js_rollup_anchor,
        js_rollup_anchor::ResponseMessage::JsResponse
    };

    type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    struct Contracts {
        lucky_raffle_account_id: AccountId,
        dapps_staking_developer_account_id: AccountId,
        reward_manager_account_id: AccountId,
    }

    async fn alice_instantiates_contract(
        client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
    ) -> Contracts {
        let reward_manager_constructor = reward_manager::ContractRef::new();
        let reward_manager_account_id = client
            .instantiate(
                "reward_manager",
                &ink_e2e::alice(),
                reward_manager_constructor,
                0,
                None,
            )
            .await
            .expect("instantiate failed")
            .account_id;

        let dapps_staking_developer_constructor = dapps_staking_developer::ContractRef::new();
        let dapps_staking_developer_account_id = client
            .instantiate(
                "dapps_staking_developer",
                &ink_e2e::alice(),
                dapps_staking_developer_constructor,
                0,
                None,
            )
            .await
            .expect("instantiate failed")
            .account_id;

        let lucky_raffle_constructor = raffle_contract::ContractRef::new(dapps_staking_developer_account_id, reward_manager_account_id);
        let lucky_raffle_account_id = client
            .instantiate(
                "lucky_raffle",
                &ink_e2e::alice(),
                lucky_raffle_constructor,
                0,
                None,
            )
            .await
            .expect("instantiate failed")
            .account_id;

        Contracts {lucky_raffle_account_id, dapps_staking_developer_account_id, reward_manager_account_id}
    }

    async fn alice_configure_contracts(
        client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
        contracts: &Contracts,
    ) {
        let grant_whitelisted_role = build_message::<dapps_staking_developer::ContractRef>(contracts.dapps_staking_developer_account_id.clone())
            .call(|contract| contract.grant_role(WHITELISTED_ADDRESS, Some(contracts.lucky_raffle_account_id)));
        client
            .call(&ink_e2e::alice(), grant_whitelisted_role, 0, None)
            .await
            .expect("grant whitelisted role failed");

        let grant_reward_manager_role = build_message::<reward_manager::ContractRef>(contracts.reward_manager_account_id.clone())
            .call(|contract| contract.grant_role(REWARD_MANAGER_ROLE, Some(contracts.lucky_raffle_account_id)));
        client
            .call(&ink_e2e::alice(), grant_reward_manager_role, 0, None)
            .await
            .expect("grant reward manager role failed");

        let set_ratio_distribution = build_message::<raffle_contract::ContractRef>(contracts.lucky_raffle_account_id.clone())
            .call(|contract| contract.set_ratio_distribution(vec![10], 100));
        client
            .call(&ink_e2e::alice(), set_ratio_distribution, 0, None)
            .await
            .expect("set ratio distribution failed");
    }

    async fn alice_set_js_script_hash(
        client: &mut ink_e2e::Client<PolkadotConfig, DefaultEnvironment>,
        contract_id: &AccountId,
    ) {
        let code_hash = [1u8; 32];
        let set_js_script_hash = build_message::<raffle_contract::ContractRef>(contract_id.clone())
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
        let set_settings_hash = build_message::<raffle_contract::ContractRef>(contract_id.clone())
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
        let grant_role = build_message::<raffle_contract::ContractRef>(contract_id.clone())
            .call(|oracle| oracle.grant_role(ATTESTOR_ROLE, Some(bob_address)));
        client
            .call(&ink_e2e::alice(), grant_role, 0, None)
            .await
            .expect("grant bob as attestor failed");
    }

    #[ink_e2e::test(additional_contracts = "contracts/lucky_raffle/Cargo.toml contracts/reward_manager/Cargo.toml contracts/dapps_staking_developer/Cargo.toml")]
    async fn test_receive_data(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // given
        let contracts = alice_instantiates_contract(&mut client).await;
        let contract_id = contracts.lucky_raffle_account_id;

        // grants the contracts
        alice_configure_contracts(&mut client, &contracts).await;

        // fund the developer contract
        let fund_dev_contract = build_message::<dapps_staking_developer::ContractRef>(contracts.dapps_staking_developer_account_id.clone())
            .call(|contract| contract.fund());
        client
            .call(&ink_e2e::alice(), fund_dev_contract, 100, None)
            .await
            .expect("fund dev contract failed");

        // check the balance of the developer contract
        let dev_contract_balance = client
            .balance(contracts.dapps_staking_developer_account_id)
            .await
            .expect("getting dev contract balance failed");

        assert_eq!(1000000100, dev_contract_balance);

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
            rewards: 100,
            winners: [dave_address].to_vec(),
        };

        let payload = JsResponse {
            js_script_hash: [1u8; 32],
            input_hash: [3u8; 32],
            settings_hash: [2u8; 32],
            output_value: response.encode(),
        };
        let actions = vec![HandleActionInput::Reply(payload.encode())];
        let rollup_cond_eq = build_message::<raffle_contract::ContractRef>(contract_id.clone())
            .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], actions.clone()));
        let result = client
            .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
            //.call_dry_run(&ink_e2e::bob(), &rollup_cond_eq, 0, None)
            .await
            .expect("rollup cond eq should be ok");
        // two events : MessageProcessedTo and ValueReceived
        assert!(result.contains_event("Contracts", "ContractEmitted"));

        //assert_eq!(result.debug_message(), "e");

        // and check if the data is filled
        let get_last_era_done = build_message::<raffle_contract::ContractRef>(contract_id.clone())
            .call(|contract| contract.get_last_era_done());
        let last_era_done = client
            .call_dry_run(&ink_e2e::charlie(), &get_last_era_done, 0, None)
            .await
            .return_value();

        assert_eq!(12, last_era_done);

        // check the balance of the developer contract
        let dev_contract_balance = client
            .balance(contracts.dapps_staking_developer_account_id)
            .await
            .expect("getting dev contract balance failed");

        assert_eq!(1000000090, dev_contract_balance);

        // check the balance of the reward manager
        let reward_manager_contract_balance = client
            .balance(contracts.reward_manager_account_id)
            .await
            .expect("getting reward manager contract balance failed");

        assert_eq!(1000000010, reward_manager_contract_balance);

        // check the balance of the raffle contract
        let raffle_contract_balance = client
            .balance(contracts.lucky_raffle_account_id)
            .await
            .expect("getting raffle contract balance failed");

        assert_eq!(1000000000, raffle_contract_balance);

        // TODO check if the user can claim rewards

        Ok(())
    }

    #[ink_e2e::test(additional_contracts = "contracts/lucky_raffle/Cargo.toml contracts/reward_manager/Cargo.toml contracts/dapps_staking_developer/Cargo.toml")]
    async fn test_receive_error(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // given
        let contracts = alice_instantiates_contract(&mut client).await;
        let contract_id = contracts.lucky_raffle_account_id;

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
        let payload = js_rollup_anchor::ResponseMessage::Error {
            js_script_hash: [1u8; 32],
            input_value: input_data.encode(),
            settings_hash: [2u8; 32],
            error,
        };
        let actions = vec![HandleActionInput::Reply(payload.encode())];
        let rollup_cond_eq = build_message::<raffle_contract::ContractRef>(contract_id.clone())
            .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], actions.clone()));
        let result = client
            .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
            .await
            .expect("we should proceed error message");
        // two events : MessageProcessedTo and ErrorReceived
        assert!(result.contains_event("Contracts", "ContractEmitted"));

        Ok(())
    }

    #[ink_e2e::test(additional_contracts = "contracts/lucky_raffle/Cargo.toml contracts/reward_manager/Cargo.toml contracts/dapps_staking_developer/Cargo.toml")]
    async fn test_bad_attestor(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // given
        let contracts = alice_instantiates_contract(&mut client).await;
        let contract_id = contracts.lucky_raffle_account_id;

        // set the js code hash
        alice_set_js_script_hash(&mut client, &contract_id).await;

        // set the settings code hash
        alice_set_settings_hash(&mut client, &contract_id).await;

        // bob is not granted as attestor => it should not be able to send a message
        let rollup_cond_eq = build_message::<raffle_contract::ContractRef>(contract_id.clone())
            .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], vec![]));
        let result = client.call(&ink_e2e::bob(), rollup_cond_eq, 0, None).await;
        assert!(
            result.is_err(),
            "only attestor should be able to send messages"
        );

        // bob is granted as attestor
        alice_grants_bob_as_attestor(&mut client, &contract_id).await;

        // then bob is able to send a message
        let rollup_cond_eq = build_message::<raffle_contract::ContractRef>(contract_id.clone())
            .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], vec![]));
        let result = client
            .call(&ink_e2e::bob(), rollup_cond_eq, 0, None)
            .await
            .expect("rollup cond eq failed");
        // no event
        assert!(!result.contains_event("Contracts", "ContractEmitted"));

        Ok(())
    }

    #[ink_e2e::test(additional_contracts = "contracts/lucky_raffle/Cargo.toml contracts/reward_manager/Cargo.toml contracts/dapps_staking_developer/Cargo.toml")]
    async fn test_bad_hash(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // given
        let contracts = alice_instantiates_contract(&mut client).await;
        let contract_id = contracts.lucky_raffle_account_id;

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
        let rollup_cond_eq = build_message::<raffle_contract::ContractRef>(contract_id.clone())
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
        let rollup_cond_eq = build_message::<raffle_contract::ContractRef>(contract_id.clone())
            .call(|oracle| oracle.rollup_cond_eq(vec![], vec![], actions.clone()));
        let result = client.call(&ink_e2e::bob(), rollup_cond_eq, 0, None).await;
        assert!(
            result.is_err(),
            "We should not accept response with bad settings code hash"
        );

        Ok(())
    }

    #[ink_e2e::test(additional_contracts = "contracts/lucky_raffle/Cargo.toml contracts/reward_manager/Cargo.toml contracts/dapps_staking_developer/Cargo.toml")]
    async fn test_bad_messages(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // given
        let contracts = alice_instantiates_contract(&mut client).await;
        let contract_id = contracts.lucky_raffle_account_id;

        // set the js code hash
        alice_set_js_script_hash(&mut client, &contract_id).await;

        // set the settings code hash
        alice_set_settings_hash(&mut client, &contract_id).await;

        // bob is granted as attestor
        alice_grants_bob_as_attestor(&mut client, &contract_id).await;

        let actions = vec![HandleActionInput::Reply(58u128.encode())];
        let rollup_cond_eq = build_message::<raffle_contract::ContractRef>(contract_id.clone())
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
    #[ink_e2e::test(additional_contracts = "contracts/lucky_raffle/Cargo.toml contracts/reward_manager/Cargo.toml contracts/dapps_staking_developer/Cargo.toml")]
    async fn test_meta_tx_rollup_cond_eq(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        let contracts = alice_instantiates_contract(&mut client).await;
        let contract_id = contracts.lucky_raffle_account_id;

        // Bob is the attestor
        // use the ecsda account because we are not able to verify the sr25519 signature
        let from = ink::primitives::AccountId::from(
            Signer::<PolkadotConfig>::account_id(&subxt_signer::ecdsa::dev::bob()).0,
        );

        // add the role => it should be succeed
        let grant_role = build_message::<raffle_contract::ContractRef>(contract_id.clone())
            .call(|oracle| oracle.grant_role(ATTESTOR_ROLE, Some(from)));
        client
            .call(&ink_e2e::alice(), grant_role, 0, None)
            .await
            .expect("grant the attestor failed");

        // prepare the meta transaction
        let data = RollupCondEqMethodParams::encode(&(vec![], vec![], vec![]));
        let prepare_meta_tx = build_message::<raffle_contract::ContractRef>(contract_id.clone())
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
            build_message::<raffle_contract::ContractRef>(contract_id.clone())
                .call(|oracle| oracle.meta_tx_rollup_cond_eq(request.clone(), signature));
        client
            .call(&ink_e2e::charlie(), meta_tx_rollup_cond_eq, 0, None)
            .await
            .expect("meta tx rollup cond eq should not failed");

        // do it again => it must failed
        let meta_tx_rollup_cond_eq =
            build_message::<raffle_contract::ContractRef>(contract_id.clone())
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