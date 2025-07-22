#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod dapps_staking_developer {

    use inkv5_client_lib::only_role;
    use inkv5_client_lib::traits::access_control::*;

    pub const WHITELISTED_ADDRESS: RoleType = ink::selector_id!("WHITELISTED_ADDRESS");

    /// Errors occurred in the contract
    #[derive(Debug, Eq, PartialEq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[allow(clippy::cast_possible_truncation)]
    pub enum ContractError {
        AccessControlError(AccessControlError),
        TransferError,
        UpgradeError,
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
        access_control: AccessControlData,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            // set the admin of this contract
            BaseAccessControl::init_with_admin(&mut instance, caller);
            BaseAccessControl::inner_grant_role(&mut instance, WHITELISTED_ADDRESS, caller)
                .expect("Should grant the role WHITELISTED_ADDRESS");
            instance
        }

        #[ink(message, payable)]
        pub fn fund(&mut self) -> Result<(), ContractError> {
            Ok(())
        }

        #[ink(message, selector = 0x410fcc9d)]
        pub fn withdraw(&mut self, value: Balance) -> Result<(), ContractError> {
            only_role!(self, WHITELISTED_ADDRESS);
            let caller = Self::env().caller();
            self.env()
                .transfer(caller, value)
                .map_err(|_| ContractError::TransferError)?;
            Ok(())
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
        pub fn get_role_whitelisted_address(&self) -> RoleType {
            WHITELISTED_ADDRESS
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
