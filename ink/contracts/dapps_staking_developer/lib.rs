#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[openbrush::implementation(AccessControl)]
#[openbrush::contract]
pub mod dapps_staking_developer {

    use openbrush::contracts::access_control::*;
    use openbrush::modifiers;
    use openbrush::traits::Storage;

    const WHITELISTED_ADDRESS: RoleType = ink::selector_id!("WHITELISTED_ADDRESS");

    /// Errors occurred in the contract
    #[derive(Debug, Eq, PartialEq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractError {
        AccessControlError(AccessControlError),
        TransferError,
        UpgradeError,
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
        access: access_control::Data,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            // set the admin of this contract
            access_control::Internal::_init_with_admin(&mut instance, Some(caller));
            AccessControl::grant_role(&mut instance, WHITELISTED_ADDRESS, Some(caller))
                .expect("Should grant the role WHITELISTED_ADDRESS");
            instance
        }

        #[ink(message, selector = 0x410fcc9d)]
        #[openbrush::modifiers(only_role(WHITELISTED_ADDRESS))]
        pub fn withdraw(&mut self, value: Balance) -> Result<(), ContractError> {
            let caller = Self::env().caller();
            self.env()
                .transfer(caller, value)
                .map_err(|_| ContractError::TransferError)?;
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn upgrade_contract(&mut self, new_code_hash: Hash) -> Result<(), ContractError> {
            self.env().set_code_hash(&new_code_hash).map_err(|_| ContractError::UpgradeError)?;
            Ok(())
        }

        #[ink(message)]
        #[modifiers(only_role(DEFAULT_ADMIN_ROLE))]
        pub fn terminate_me(&mut self) -> Result<(), ContractError> {
            self.env().terminate_contract(self.env().caller());
        }

        #[ink(message)]
        pub fn get_role_whitelisted_address(&self) -> RoleType {
            WHITELISTED_ADDRESS
        }
    }
}
