#![cfg_attr(not(feature = "std"), no_std, no_main)]

extern crate alloc;
extern crate core;

#[ink::contract(env = pink_extension::PinkEnvironment)]
mod dapp_staking {

    use alloc::{string::String, vec::Vec};
    use pink_extension::chain_extension::signing;
    use pink_extension::error;
    use scale::{Decode, Encode};

    #[derive(Encode, Decode, Debug)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    struct Config {
        /// The RPC endpoint of the target blockchain
        rpc: String,
        pallet_id: u8,
        call_id: u8,
        smart_contract: AccountId,
        /// Key for sending out the meta-tx. None to fallback to the wallet based auth.
        sender_key: Option<[u8; 32]>,
    }

    #[ink(storage)]
    pub struct DappStaking {
        owner: AccountId,
        /// config to send the data to the ink! smart contract
        config: Option<Config>,
        /// Key for signing the tx if no sender key is defined in the config
        private_key: [u8; 32],
    }

    #[derive(Encode, Decode, Debug)]
    #[repr(u8)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractError {
        BadOrigin,
        NotConfigured,
        InvalidKeyLength,
        SubRpcError,
    }

    impl From<subrpc::traits::common::Error> for ContractError {
        fn from(error: subrpc::traits::common::Error) -> Self {
            error!("error in the subrpc: {:?}", error);
            ContractError::SubRpcError
        }
    }

    #[derive(Encode, Decode, PartialEq, Eq, Clone, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ClaimDappRewardCall {
        smart_contract: SmartContract,
        #[codec(compact)]
        era: u32,
    }

    #[derive(Encode, Decode, PartialEq, Eq, Clone, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum SmartContract {
        /// EVM smart contract : It's a 20 byte representation
        Evm([u8; 20]),
        /// Wasm smart contract
        Wasm(AccountId),
    }

    type Result<T> = core::result::Result<T, ContractError>;

    impl DappStaking {
        #[ink(constructor)]
        pub fn default() -> Self {
            const NONCE: &[u8] = b"attest_key";
            let private_key = signing::derive_sr25519_key(NONCE);

            Self {
                owner: Self::env().caller(),
                config: None,
                private_key: private_key[..32].try_into().expect("Invalid Key Length"),
            }
        }

        /// Gets the owner of the contract
        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        /// Gets the sender address used by this rollup
        #[ink(message)]
        pub fn get_sender_address(&self) -> Vec<u8> {
            if let Some(Some(sender_key)) = self.config.as_ref().map(|c| c.sender_key.as_ref()) {
                signing::get_public_key(sender_key, signing::SigType::Sr25519)
            } else {
                signing::get_public_key(&self.private_key, signing::SigType::Sr25519)
            }
        }

        /// Gets the config for the call
        #[ink(message)]
        pub fn get_call(&self) -> Option<(String, u8, u8, AccountId)> {
            self.config
                .as_ref()
                .map(|c| (c.rpc.clone(), c.pallet_id, c.call_id, c.smart_contract))
        }

        /// Configures the call (admin only)
        #[ink(message)]
        pub fn config_call(
            &mut self,
            rpc: String,
            pallet_id: u8,
            call_id: u8,
            smart_contract: AccountId,
            sender_key: Option<Vec<u8>>,
        ) -> Result<()> {
            self.ensure_owner()?;
            self.config = Some(Config {
                rpc,
                pallet_id,
                call_id,
                smart_contract,
                sender_key: match sender_key {
                    Some(key) => Some(key.try_into().or(Err(ContractError::InvalidKeyLength))?),
                    None => None,
                },
            });
            Ok(())
        }

        /// Transfers the ownership of the contract (admin only)
        #[ink(message)]
        pub fn transfer_ownership(&mut self, new_owner: AccountId) -> Result<()> {
            self.ensure_owner()?;
            self.owner = new_owner;
            Ok(())
        }

        /// Processes a request by a rollup transaction
        #[ink(message)]
        pub fn claim_dapp_rewards(&self, era: u32) -> Result<Option<Vec<u8>>> {
            let config = self.ensure_configured()?;

            let data = ClaimDappRewardCall {
                smart_contract: SmartContract::Wasm(config.smart_contract),
                era,
            };

            let sender_key = config.sender_key.unwrap_or(self.private_key);

            let signed_tx = subrpc::create_transaction(
                &sender_key,
                "astar",
                &config.rpc,
                config.pallet_id,
                config.call_id,
                data,
                subrpc::ExtraParam::default(),
            )?;

            let tx_id = subrpc::send_transaction(&config.rpc, &signed_tx)?;

            Ok(Some(tx_id))
        }

        /// Returns BadOrigin error if the caller is not the owner
        fn ensure_owner(&self) -> Result<()> {
            if self.env().caller() == self.owner {
                Ok(())
            } else {
                Err(ContractError::BadOrigin)
            }
        }

        /// Returns the config reference or raise the error `NotConfigured`
        fn ensure_configured(&self) -> Result<&Config> {
            self.config.as_ref().ok_or(ContractError::NotConfigured)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::debug_println;

        struct EnvVars {
            /// The RPC endpoint of the target blockchain
            rpc: String,
            pallet_id: u8,
            call_id: u8,
            /// smart contract
            smart_contract: AccountId,
            /// When we want to use meta tx
            signer_key: Option<Vec<u8>>,
        }

        fn get_env(key: &str) -> String {
            std::env::var(key).expect("env not found")
        }

        fn config() -> EnvVars {
            dotenvy::dotenv().ok();
            let rpc = get_env("RPC");
            let pallet_id: u8 = get_env("PALLET_ID").parse().expect("u8 expected");
            let call_id: u8 = get_env("CALL_ID").parse().expect("u8 expected");
            let sc: [u8; 32] = hex::decode(get_env("SMART_CONTRACT"))
                .expect("hex decode failed")
                .try_into()
                .expect("incorrect length");
            let smart_contract: AccountId = sc.into();
            let signer_key = std::env::var("SIGNER_KEY")
                .map(|s| hex::decode(s).expect("hex decode failed"))
                .ok();

            EnvVars {
                rpc: rpc.to_string(),
                pallet_id,
                call_id,
                smart_contract: smart_contract.into(),
                signer_key,
            }
        }

        fn init_contract() -> DappStaking {
            let EnvVars {
                rpc,
                pallet_id,
                call_id,
                smart_contract,
                signer_key,
            } = config();

            let mut contract = DappStaking::default();
            contract
                .config_call(rpc, pallet_id, call_id, smart_contract.into(), signer_key)
                .unwrap();

            contract
        }

        #[ink::test
        #[ignore = "Update the era before running this test"]]
        fn claim_dapp_rewards() {
            let _ = env_logger::try_init();
            pink_extension_runtime::mock_ext::mock_all_ext();

            let contract = init_contract();

            let r = contract
                .claim_dapp_rewards(4522)
                .expect("failed to answer request");
            debug_println!("answer request: {r:?}");
        }
    }
}
