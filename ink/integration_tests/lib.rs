#![cfg_attr(not(feature = "std"), no_std, no_main)]
#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests {
    use core::fmt::Debug;
    use ink::env::DefaultEnvironment;
    use ink_e2e::{ChainBackend, ContractsBackend, E2EBackend, InstantiationResult};
    use ink::scale::Encode;
    use ink::primitives::AccountId;

    use lucky::traits::raffle::*;
    use lucky::traits::reward::psp22_reward::*;
    use dapps_staking_developer::{dapps_staking_developer, *};
    use reward_manager::{reward_manager};
    use raffle_consumer::{RaffleResponseMessage, raffle_consumer};

    use inkv5_client_lib::traits::access_control::*;
    use inkv5_client_lib::traits::meta_transaction::*;
    use inkv5_client_lib::traits::rollup_client::*;

    type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    async fn alice_instantiates_reward_manager_contract<Client>(
        client: &mut Client,
    ) -> InstantiationResult<
        DefaultEnvironment,
        <Client as ContractsBackend<DefaultEnvironment>>::EventLog,
    >
    where
        Client: E2EBackend,
        <Client as ContractsBackend<DefaultEnvironment>>::Error: Debug,
    {
        let mut reward_manager_constructor = reward_manager::ContractRef::new();
        let reward_manager_contract = client
            .instantiate(
                "reward_manager",
                &ink_e2e::alice(),
                &mut reward_manager_constructor,
            )
            .submit()
            .await
            .expect("instantiate failed");

        reward_manager_contract
    }


    async fn alice_instantiates_dapps_staking_developer_contract<Client>(
        client: &mut Client,
    ) -> InstantiationResult<
        DefaultEnvironment,
        <Client as ContractsBackend<DefaultEnvironment>>::EventLog,
    >
    where
        Client: E2EBackend,
        <Client as ContractsBackend<DefaultEnvironment>>::Error: Debug,
    {
        let mut dapps_staking_developer_constructor = dapps_staking_developer::ContractRef::new();
        let dapps_staking_developer_contract = client
            .instantiate(
                "dapps_staking_developer",
                &ink_e2e::alice(),
                &mut dapps_staking_developer_constructor,
            )
            .submit()
            .await
            .expect("instantiate failed");

        dapps_staking_developer_contract
    }


    async fn alice_instantiates_raffle_consumer_contract<Client>(
        client: &mut Client,
        dapps_staking_developer_account_id: AccountId,
        reward_manager_account_id: AccountId,
    ) -> InstantiationResult<
        DefaultEnvironment,
        <Client as ContractsBackend<DefaultEnvironment>>::EventLog,
    >
    where
        Client: E2EBackend,
        <Client as ContractsBackend<DefaultEnvironment>>::Error: Debug,
    {
        let mut raffle_consumer_constructor = raffle_consumer::ContractRef::new(
            dapps_staking_developer_account_id,
            reward_manager_account_id,
        );
        let raffle_consumer_contract = client
            .instantiate(
                "raffle_consumer",
                &ink_e2e::alice(),
                &mut raffle_consumer_constructor,
            )
            .submit()
            .await
            .expect("instantiate failed");

        raffle_consumer_contract
    }

    async fn alice_configure_contracts<Client>(
        client: &mut Client,
        reward_manager_contract: &InstantiationResult<
            DefaultEnvironment,
            <Client as ContractsBackend<DefaultEnvironment>>::EventLog,
        >,
        dapps_staking_developer_contract: &InstantiationResult<
            DefaultEnvironment,
            <Client as ContractsBackend<DefaultEnvironment>>::EventLog,
        >,
        raffle_consumer_contract: &InstantiationResult<
            DefaultEnvironment,
            <Client as ContractsBackend<DefaultEnvironment>>::EventLog,
        >,
    ) where
        Client: E2EBackend,
        <Client as ContractsBackend<DefaultEnvironment>>::Error: Debug,
    {
        let grant_whitelisted_role = dapps_staking_developer_contract
            .call_builder::<dapps_staking_developer::Contract>()
            .grant_role(WHITELISTED_ADDRESS, raffle_consumer_contract.account_id);
        client
            .call(&ink_e2e::alice(), &grant_whitelisted_role)
            .submit()
            .await
            .expect("grant whitelisted role failed");

        let grant_reward_manager_role = reward_manager_contract
            .call_builder::<reward_manager::Contract>()
            .grant_role(REWARD_MANAGER_ROLE, raffle_consumer_contract.account_id);

        client
            .call(&ink_e2e::alice(), &grant_reward_manager_role)
            .submit()
            .await
            .expect("grant reward manager role failed");

        let set_ratio_distribution = raffle_consumer_contract
            .call_builder::<raffle_consumer::Contract>()
            .set_ratio_distribution(vec![10], 100);

        client
            .call(&ink_e2e::alice(), &set_ratio_distribution)
            .submit()
            .await
            .expect("set ratio distribution failed");

        let set_next_era = raffle_consumer_contract
            .call_builder::<raffle_consumer::Contract>()
            .set_next_era(13);

        client
            .call(&ink_e2e::alice(), &set_next_era)
            .submit()
            .await
            .expect("set last era failed");

    }

    async fn alice_grants_bob_as_attestor<Client>(
        client: &mut Client,
        contract: &InstantiationResult<
            DefaultEnvironment,
            <Client as ContractsBackend<DefaultEnvironment>>::EventLog,
        >,
    ) where
        Client: E2EBackend,
        <Client as ContractsBackend<DefaultEnvironment>>::Error: Debug,
    {
        // bob is granted as attestor
        let bob_address = ink::primitives::AccountId::from(ink_e2e::bob().public_key().0);
        let grant_role = contract
            .call_builder::<raffle_consumer::Contract>()
            .grant_role(ATTESTOR_ROLE, bob_address);
        client
            .call(&ink_e2e::alice(), &grant_role)
            .submit()
            .await
            .expect("grant bob as attestor failed");
    }

    #[ink_e2e::test]
    async fn test_do_raffle<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
        // given
        let reward_manager_contract = alice_instantiates_reward_manager_contract(&mut client).await;
        let dapps_staking_developer_contract = alice_instantiates_dapps_staking_developer_contract(&mut client).await;
        let raffle_consumer_contract = alice_instantiates_raffle_consumer_contract(
            &mut client,
            dapps_staking_developer_contract.account_id,
            reward_manager_contract.account_id,
        ).await;

        // configure the contracts
        alice_configure_contracts(
            &mut client,
            &reward_manager_contract,
            &dapps_staking_developer_contract,
            &raffle_consumer_contract
        ).await;

        // fund the developer contract
        let fund_dev_contract = dapps_staking_developer_contract
            .call_builder::<dapps_staking_developer::Contract>()
            .fund();

        client
            .call(&ink_e2e::alice(), &fund_dev_contract)
            .value(100)
            .submit()
            .await
            .expect("fund dev contract failed");

        // check the balance of the developer contract
        let dev_contract_balance = client
            .free_balance(dapps_staking_developer_contract.account_id)
            .await
            .expect("getting dev contract balance failed");

        assert_eq!(1000000100, dev_contract_balance);

        // bob is granted as attestor
        alice_grants_bob_as_attestor(&mut client, &raffle_consumer_contract).await;

        let dave_address = ink::primitives::AccountId::from(ink_e2e::dave().public_key().0);

        // data is received
        let response = RaffleResponseMessage {
            era: 13,
            skipped: false,
            rewards: 100,
            winners: [dave_address].to_vec(),
        };

        let actions = vec![HandleActionInput::Reply(response.encode())];
        let rollup_cond_eq = raffle_consumer_contract
            .call_builder::<raffle_consumer::Contract>()
            .rollup_cond_eq(vec![], vec![], actions.clone());
/*
           let result = client.call(&ink_e2e::bob(), &rollup_cond_eq).dry_run().await.expect("dry_run should be ok");
           assert_eq!(
               result.debug_message(),
               "only attestor should be able to send messages"
           );

 */

        let result = client
            .call(&ink_e2e::bob(), &rollup_cond_eq)
            .submit()
            .await
            .expect("rollup cond eq should be ok");

        // two events : MessageProcessedTo and RaffleDone
        assert!(result.contains_event("Contracts", "ContractEmitted"));

        // test wrong era => meaning only 1 raffle by era
        let result = client.call(&ink_e2e::bob(), &rollup_cond_eq).submit().await;
        assert!(result.is_err(), "Era must be sequential without blank");

        // and check if the data is filled
        let get_next_era = raffle_consumer_contract
            .call_builder::<raffle_consumer::Contract>()
            .get_next_era();

        let next_era = client
            .call(&ink_e2e::charlie(), &get_next_era)
            .dry_run()
            .await
            .expect("fail to get next era")
            .return_value()
            .expect("next era failed");

        assert_eq!(14, next_era);

        // check the balance of the developer contract
        let dev_contract_balance = client
            .free_balance(dapps_staking_developer_contract.account_id)
            .await
            .expect("getting dev contract balance failed");

        assert_eq!(1000000090, dev_contract_balance);

        // check the balance of the reward manager
        let reward_manager_contract_balance = client
            .free_balance(reward_manager_contract.account_id)
            .await
            .expect("getting reward manager contract balance failed");

        assert_eq!(1000000010, reward_manager_contract_balance);

        // check the balance of the raffle contract
        let raffle_consumer_balance = client
            .free_balance(raffle_consumer_contract.account_id)
            .await
            .expect("getting raffle contract balance failed");

        assert_eq!(1000000000, raffle_consumer_balance);

        // check the balance of dave
        let dave_balance_before_claim = client
            .free_balance(dave_address)
            .await
            .expect("getting Dave balance failed");

        let claim = reward_manager_contract
            .call_builder::<reward_manager::Contract>()
            .claim();

        let result = client
            .call(&ink_e2e::dave(), &claim)
            .submit()
            .await
            .expect("Claim rewards should be ok");
        // 1 event : RewardsClaimed
        assert!(result.contains_event("Contracts", "ContractEmitted"));

        // check the balance of dave
        let dave_balance_after_claim = client
            .free_balance(dave_address)
            .await
            .expect("getting Dave balance failed");

        // we cannot calculate the balance because of fees
        assert!(dave_balance_after_claim > dave_balance_before_claim);

        // check the balance of the reward manager
        let reward_manager_contract_balance_after_claim = client
            .free_balance(reward_manager_contract.account_id)
            .await
            .expect("getting reward manager contract balance failed");

        assert_eq!(
            reward_manager_contract_balance_after_claim,
            reward_manager_contract_balance - 10
        );

        Ok(())
    }

    #[ink_e2e::test]
    async fn test_skip_raffle<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
        // given
        let reward_manager_contract = alice_instantiates_reward_manager_contract(&mut client).await;
        let dapps_staking_developer_contract = alice_instantiates_dapps_staking_developer_contract(&mut client).await;
        let raffle_consumer_contract = alice_instantiates_raffle_consumer_contract(
            &mut client,
            dapps_staking_developer_contract.account_id,
            reward_manager_contract.account_id,
        ).await;

        // configure the contracts
        alice_configure_contracts(
            &mut client,
            &reward_manager_contract,
            &dapps_staking_developer_contract,
            &raffle_consumer_contract
        ).await;

        // bob is granted as attestor
        alice_grants_bob_as_attestor(&mut client, &raffle_consumer_contract).await;

        // data is received
        let response = RaffleResponseMessage {
            era: 13,
            skipped: true,
            rewards: 0,
            winners: [].to_vec(),
        };

        let actions = vec![HandleActionInput::Reply(response.encode())];
        let rollup_cond_eq = raffle_consumer_contract
            .call_builder::<raffle_consumer::Contract>()
            .rollup_cond_eq(vec![], vec![], actions.clone());

        let result = client
            .call(&ink_e2e::bob(), &rollup_cond_eq)
            .submit()
            .await
            .expect("rollup cond eq should be ok");
        // two events : MessageProcessedTo and RaffleSkipped
        assert!(result.contains_event("Contracts", "ContractEmitted"));

        // and check if the data is filled
        let get_next_era = raffle_consumer_contract
            .call_builder::<raffle_consumer::Contract>()
            .get_next_era();

        let next_era = client
            .call(&ink_e2e::charlie(), &get_next_era)
            .dry_run()
            .await
            .expect("fail to get next era")
            .return_value()
            .expect("next era failed");

        assert_eq!(14, next_era);

        // test wrong era => meaning only 1 raffle by era
        let result = client.call(&ink_e2e::bob(), &rollup_cond_eq).submit().await;
        assert!(result.is_err(), "Era must be sequential without blank");

        Ok(())
    }


    #[ink_e2e::test]
    async fn test_bad_attestor<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
        // given
        let reward_manager_contract = alice_instantiates_reward_manager_contract(&mut client).await;
        let dapps_staking_developer_contract = alice_instantiates_dapps_staking_developer_contract(&mut client).await;
        let raffle_consumer_contract = alice_instantiates_raffle_consumer_contract(
            &mut client,
            dapps_staking_developer_contract.account_id,
            reward_manager_contract.account_id,
        ).await;

        // configure the contracts
        alice_configure_contracts(
            &mut client,
            &reward_manager_contract,
            &dapps_staking_developer_contract,
            &raffle_consumer_contract
        ).await;

        // bob is not granted as attestor => it should not be able to send a message
        let rollup_cond_eq = raffle_consumer_contract
            .call_builder::<raffle_consumer::Contract>()
            .rollup_cond_eq(vec![], vec![], vec![]);
        let result = client.call(&ink_e2e::bob(), &rollup_cond_eq).submit().await;
        assert!(
            result.is_err(),
            "only attestor should be able to send messages"
        );

        // bob is granted as attestor
        alice_grants_bob_as_attestor(&mut client, &raffle_consumer_contract).await;

        // then bob is able to send a message
        let result = client
            .call(&ink_e2e::bob(), &rollup_cond_eq)
            .submit()
            .await
            .expect("rollup cond eq failed");
        // no event
        assert!(!result.contains_event("Contracts", "ContractEmitted"));

        Ok(())
    }


    #[ink_e2e::test]
    async fn test_bad_messages(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // given
        let reward_manager_contract = alice_instantiates_reward_manager_contract(&mut client).await;
        let dapps_staking_developer_contract = alice_instantiates_dapps_staking_developer_contract(&mut client).await;
        let raffle_consumer_contract = alice_instantiates_raffle_consumer_contract(
            &mut client,
            dapps_staking_developer_contract.account_id,
            reward_manager_contract.account_id,
        ).await;

        // configure the contracts
        alice_configure_contracts(
            &mut client,
            &reward_manager_contract,
            &dapps_staking_developer_contract,
            &raffle_consumer_contract
        ).await;

        // bob is granted as attestor
        alice_grants_bob_as_attestor(&mut client, &raffle_consumer_contract).await;

        let actions = vec![HandleActionInput::Reply(58u128.encode())];
        let rollup_cond_eq = raffle_consumer_contract
            .call_builder::<raffle_consumer::Contract>()
            .rollup_cond_eq(vec![], vec![], actions.clone());
        let result = client.call(&ink_e2e::bob(), &rollup_cond_eq).submit().await;
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

        // given
        let reward_manager_contract = alice_instantiates_reward_manager_contract(&mut client).await;
        let dapps_staking_developer_contract = alice_instantiates_dapps_staking_developer_contract(&mut client).await;
        let raffle_consumer_contract = alice_instantiates_raffle_consumer_contract(
            &mut client,
            dapps_staking_developer_contract.account_id,
            reward_manager_contract.account_id,
        ).await;

        // configure the contracts
        alice_configure_contracts(
            &mut client,
            &reward_manager_contract,
            &dapps_staking_developer_contract,
            &raffle_consumer_contract
        ).await;

        // Bob is the attestor
        // use the ecsda account because we are not able to verify the sr25519 signature
        let bob_keypair = subxt_signer::ecdsa::dev::bob();
        let from = ink::primitives::AccountId::from(bob_keypair.public_key().to_account_id().0);

        // add the role => it should succeed
        let grant_role = raffle_consumer_contract
            .call_builder::<raffle_consumer::Contract>()
            .grant_role(ATTESTOR_ROLE, from);
        client
            .call(&ink_e2e::alice(), &grant_role)
            .submit()
            .await
            .expect("grant the attestor failed");

        // prepare the meta transaction
        let data = RollupCondEqMethodParams::encode(&(vec![], vec![], vec![]));
        let prepare_meta_tx = raffle_consumer_contract
            .call_builder::<raffle_consumer::Contract>()
            .prepare(from, data.clone());
        let result = client
            .call(&ink_e2e::charlie(), &prepare_meta_tx)
            .dry_run()
            .await
            .expect("We should be able to prepare the meta tx");

        let (request, _hash) = result
            .return_value()
            .expect("Expected value when preparing meta tx");

        assert_eq!(0, request.nonce);
        assert_eq!(from, request.from);
        assert_eq!(&data, &request.data);

        // Bob signs the message
        let signature = bob_keypair.sign(&ink::scale::Encode::encode(&request)).0;

        // do the meta tx: charlie sends the message
        let meta_tx_rollup_cond_eq = raffle_consumer_contract
            .call_builder::<raffle_consumer::Contract>()
            .meta_tx_rollup_cond_eq(request.clone(), signature);
        client
            .call(&ink_e2e::charlie(), &meta_tx_rollup_cond_eq)
            .submit()
            .await
            .expect("meta tx rollup cond eq should not failed");

        // do it again => it must fail
        let result = client
            .call(&ink_e2e::charlie(), &meta_tx_rollup_cond_eq)
            .submit()
            .await;
        assert!(
            result.is_err(),
            "This message should not be proceed because the nonce is obsolete"
        );

        Ok(())
    }


}
