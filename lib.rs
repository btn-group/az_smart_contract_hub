#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod az_smart_contract_hub {
    use ink::prelude::string::{String, ToString};
    use ink::storage::Mapping;

    // === ENUMS ===
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum AZSmartContractHubError {
        NotFound(String),
        OutOfRange(String),
        Unauthorised,
        Unchanged(String),
    }

    // === EVENTS ===
    #[ink(event)]
    pub struct Create {
        id: u32,
        smart_contract_address: AccountId,
        url: String,
        environment: u8,
        caller: AccountId,
    }

    #[ink(event)]
    pub struct Toggle {
        id: u32,
        enabled: bool,
    }

    #[ink(event)]
    pub struct Rate {
        id: u32,
        previous_user_rating: i8,
        new_user_rating: i8,
        user: AccountId,
    }

    // === STRUCTS ===
    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct SmartContract {
        id: u32,
        smart_contract_address: AccountId,
        url: String,
        environment: u8,
        likes: u16,
        dislikes: u16,
        caller: AccountId,
        enabled: bool,
    }

    // === CONTRACT ===
    #[ink(storage)]
    #[derive(Default)]
    pub struct AZSmartContractHub {
        smart_contracts: Mapping<u32, SmartContract>,
        smart_contracts_count: u32,
        user_ratings: Mapping<(u32, AccountId), i8>,
    }
    impl AZSmartContractHub {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                smart_contracts: Mapping::default(),
                smart_contracts_count: 0,
                user_ratings: Mapping::default(),
            }
        }

        // === QUERIES ===
        #[ink(message)]
        pub fn show(&self, id: u32) -> Result<SmartContract, AZSmartContractHubError> {
            if let Some(smart_contract) = self.smart_contracts.get(id) {
                Ok(smart_contract)
            } else {
                Err(AZSmartContractHubError::NotFound(
                    "SmartContract".to_string(),
                ))
            }
        }

        // === HANDLES ===
        // 0 == Production
        // 1 == Testnet
        // 2 == Smarknet
        #[ink(message)]
        pub fn create(
            &mut self,
            smart_contract_address: AccountId,
            url: String,
            environment: u8,
        ) -> Result<SmartContract, AZSmartContractHubError> {
            let caller: AccountId = Self::env().caller();
            let smart_contract: SmartContract = SmartContract {
                id: self.smart_contracts_count,
                smart_contract_address,
                url: url.clone(),
                environment,
                likes: 1,
                dislikes: 0,
                caller: Self::env().caller(),
                enabled: true,
            };
            self.smart_contracts
                .insert(self.smart_contracts_count, &smart_contract);
            self.smart_contracts_count += 1;

            self.user_ratings.insert((smart_contract.id, caller), &1);

            // emit event
            self.env().emit_event(Create {
                id: smart_contract.id,
                smart_contract_address,
                url,
                environment,
                caller,
            });

            Ok(smart_contract)
        }

        #[ink(message)]
        pub fn rate(
            &mut self,
            id: u32,
            new_user_rating: i8,
        ) -> Result<SmartContract, AZSmartContractHubError> {
            let mut smart_contract: SmartContract = self.show(id)?;
            if !(-1..=1).contains(&new_user_rating) {
                return Err(AZSmartContractHubError::OutOfRange("Rating".to_string()));
            }

            let caller: AccountId = Self::env().caller();
            let previous_user_rating: i8 = self.user_ratings.get((id, caller)).unwrap_or(0);
            if previous_user_rating == new_user_rating {
                return Err(AZSmartContractHubError::Unchanged("Rating".to_string()));
            }

            if new_user_rating == -1 {
                if previous_user_rating == 1 {
                    smart_contract.likes -= 1
                }
                smart_contract.dislikes += 1
            } else if new_user_rating == 0 {
                if previous_user_rating == 1 {
                    smart_contract.likes -= 1
                } else if previous_user_rating == -1 {
                    smart_contract.dislikes -= 1
                }
            } else {
                if previous_user_rating == -1 {
                    smart_contract.dislikes -= 1
                }
                smart_contract.likes += 1
            }
            self.smart_contracts
                .insert(smart_contract.id, &smart_contract);
            self.user_ratings.insert((id, caller), &new_user_rating);

            // emit event
            self.env().emit_event(Rate {
                id: smart_contract.id,
                previous_user_rating,
                new_user_rating,
                user: caller,
            });

            Ok(smart_contract)
        }

        #[ink(message)]
        pub fn toggle_enabled(
            &mut self,
            id: u32,
            enabled: bool,
        ) -> Result<SmartContract, AZSmartContractHubError> {
            let mut smart_contract: SmartContract = self.show(id)?;
            let caller: AccountId = Self::env().caller();
            if caller != smart_contract.caller {
                return Err(AZSmartContractHubError::Unauthorised);
            }
            if smart_contract.enabled == enabled {
                return Err(AZSmartContractHubError::Unchanged("Enabled".to_string()));
            }

            smart_contract.enabled = enabled;
            self.smart_contracts
                .insert(smart_contract.id, &smart_contract);

            // emit event
            self.env().emit_event(Toggle {
                id: smart_contract.id,
                enabled: smart_contract.enabled,
            });

            Ok(smart_contract)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{
            test::{default_accounts, set_caller, DefaultAccounts},
            DefaultEnvironment,
        };

        const MOCK_URL: &str = "https://res.cloudinary.com/hv5cxagki/raw/upload/v1690808298/smart_contract_metadata/tmuurccd5a7lcvin6ae9.json";

        // === HELPERS ===
        fn init() -> (DefaultAccounts<DefaultEnvironment>, AZSmartContractHub) {
            let accounts = default_accounts();
            set_caller::<DefaultEnvironment>(accounts.bob);
            let az_smart_contract_hub = AZSmartContractHub::new();
            (accounts, az_smart_contract_hub)
        }

        // === TESTS ===
        // === TEST QUERIES ===
        #[ink::test]
        fn test_show() {
            let (accounts, mut az_smart_contract_hub) = init();
            // = when smart_contract does not exist
            // * it returns error
            assert_eq!(
                az_smart_contract_hub.show(0),
                Err(AZSmartContractHubError::NotFound(
                    "SmartContract".to_string()
                ))
            );
            // = when smart_contract exists
            let smart_contract: SmartContract = az_smart_contract_hub
                .create(accounts.alice, MOCK_URL.to_string(), 0)
                .unwrap();
            // = * it returns the smart_contract
            assert_eq!(
                az_smart_contract_hub.show(smart_contract.id),
                Ok(smart_contract)
            );
        }

        // === TEST HANDLES ===
        #[ink::test]
        fn test_create() {
            let (accounts, mut az_smart_contract_hub) = init();
            // when environment is within range
            az_smart_contract_hub.smart_contracts_count = u32::MAX - 1;
            // * it stores the caller as the caller
            let result = az_smart_contract_hub.create(accounts.alice, MOCK_URL.to_string(), 0);
            let result_unwrapped = result.unwrap();
            // * it stores the id as the current length
            assert_eq!(result_unwrapped.id, u32::MAX - 1);
            // * it increases the smart_contracts length by 1
            assert_eq!(az_smart_contract_hub.smart_contracts_count, u32::MAX);
            // * it stores the submitted smart contract address
            assert_eq!(result_unwrapped.smart_contract_address, accounts.alice);
            // * it sets the like to 1 and dislike to 0
            assert_eq!(result_unwrapped.likes, 1);
            assert_eq!(result_unwrapped.dislikes, 0);
            // * it sets the caller to the caller
            assert_eq!(result_unwrapped.caller, accounts.bob);
            // * it sets the user_rating to 1
            assert_eq!(
                az_smart_contract_hub
                    .user_ratings
                    .get((result_unwrapped.id, result_unwrapped.caller))
                    .unwrap(),
                1
            );
            // * it stores the smart_contract
            assert_eq!(
                result_unwrapped,
                az_smart_contract_hub.show(result_unwrapped.id).unwrap()
            );
        }

        #[ink::test]
        fn test_rate() {
            let (accounts, mut az_smart_contract_hub) = init();
            // = when smart_contract doesn't exist
            // = * it raises an error
            let mut result = az_smart_contract_hub.rate(0, 0);
            assert_eq!(
                result,
                Err(AZSmartContractHubError::NotFound(
                    "SmartContract".to_string()
                ))
            );

            // = when smart_contract exists
            az_smart_contract_hub
                .create(accounts.alice, MOCK_URL.to_string(), 1)
                .unwrap();
            // == when new rating is less than -1
            result = az_smart_contract_hub.rate(0, -2);
            // == * it raises an error
            assert_eq!(
                result,
                Err(AZSmartContractHubError::OutOfRange("Rating".to_string()))
            );
            // == when new rating is grater than 1
            result = az_smart_contract_hub.rate(0, 2);
            // == * it raises an error
            assert_eq!(
                result,
                Err(AZSmartContractHubError::OutOfRange("Rating".to_string()))
            );
            // == when new rating is within range
            // === when new rating is the same as user's current rating
            result = az_smart_contract_hub.rate(0, 1);
            // === * it raises an error
            assert_eq!(
                result,
                Err(AZSmartContractHubError::Unchanged("Rating".to_string()))
            );

            // === when new rating is different to user's current rating
            // ==== when current rating is 1
            // ===== when new rating is 0
            let mut new_user_rating: i8 = 0;
            result = az_smart_contract_hub.rate(0, new_user_rating);
            let mut smart_contract = result.unwrap();
            // ===== * it reduces the smart_contract's likes by 1
            // ===== * it does not change the dislikes
            // ===== * it sets the user's rating for the smart_contract to 1
            assert_eq!(smart_contract.likes, 0);
            assert_eq!(smart_contract.dislikes, 0);
            assert_eq!(
                az_smart_contract_hub
                    .user_ratings
                    .get((0, accounts.bob))
                    .unwrap_or(0),
                new_user_rating
            );

            // ===== when new rating is -1
            az_smart_contract_hub.rate(0, 1).unwrap();
            new_user_rating = -1;
            smart_contract = az_smart_contract_hub.rate(0, new_user_rating).unwrap();
            // ===== * it reduces the smart_contract's likes by 1
            // ===== * it increases the smart_contract's dislikes by 1
            // ===== * it sets the user's rating for the smart_contract to -1
            assert_eq!(smart_contract.likes, 0);
            assert_eq!(smart_contract.dislikes, 1);
            assert_eq!(
                az_smart_contract_hub
                    .user_ratings
                    .get((0, accounts.bob))
                    .unwrap_or(0),
                new_user_rating
            );

            // ==== when current rating is -1
            // ===== when new rating is 0
            new_user_rating = 0;
            result = az_smart_contract_hub.rate(0, new_user_rating);
            smart_contract = result.unwrap();
            // ===== * it reduces the smart_contract's dislikes by 1
            // ===== * it does not change the likes
            // ===== * it sets the user's rating for the smart_contract to 0
            assert_eq!(smart_contract.likes, 0);
            assert_eq!(smart_contract.dislikes, 0);
            assert_eq!(
                az_smart_contract_hub
                    .user_ratings
                    .get((0, accounts.bob))
                    .unwrap_or(0),
                new_user_rating
            );

            // ===== when new rating is 1
            az_smart_contract_hub.rate(0, -1).unwrap();
            new_user_rating = 1;
            result = az_smart_contract_hub.rate(0, new_user_rating);
            smart_contract = result.unwrap();
            // ===== * it reduces the smart_contract's dislikes by 1
            // ===== * it increases the likes by 1
            // ===== * it sets the user's rating for the smart_contract to 1
            assert_eq!(smart_contract.likes, 1);
            assert_eq!(smart_contract.dislikes, 0);
            assert_eq!(
                az_smart_contract_hub
                    .user_ratings
                    .get((0, accounts.bob))
                    .unwrap_or(0),
                new_user_rating
            );

            // ==== when current rating is 0
            az_smart_contract_hub.rate(0, 0).unwrap();
            // ===== when new rating is 1
            new_user_rating = 1;
            result = az_smart_contract_hub.rate(0, new_user_rating);
            smart_contract = result.unwrap();
            // ===== * it does not change the dislikes
            // ===== * it increases the likes by 1
            // ===== * it sets the user's rating for the smart_contract to 1
            assert_eq!(smart_contract.likes, 1);
            assert_eq!(smart_contract.dislikes, 0);
            assert_eq!(
                az_smart_contract_hub
                    .user_ratings
                    .get((0, accounts.bob))
                    .unwrap_or(0),
                new_user_rating
            );

            az_smart_contract_hub.rate(0, 0).unwrap();
            // ===== when new rating is -1
            new_user_rating = -1;
            result = az_smart_contract_hub.rate(0, new_user_rating);
            smart_contract = result.unwrap();
            // ===== * it increases the dislikes by 1
            // ===== * it does not change the likes
            // ===== * it sets the user's rating for the smart_contract to -1
            assert_eq!(smart_contract.likes, 0);
            assert_eq!(smart_contract.dislikes, 1);
            assert_eq!(
                az_smart_contract_hub
                    .user_ratings
                    .get((0, accounts.bob))
                    .unwrap_or(0),
                new_user_rating
            );
        }

        #[ink::test]
        fn test_toggle_enabled() {
            let (accounts, mut az_smart_contract_hub) = init();
            // = when smart_contract doesn't exist
            // = * it raises an error
            let mut result = az_smart_contract_hub.toggle_enabled(0, false);
            assert_eq!(
                result,
                Err(AZSmartContractHubError::NotFound(
                    "SmartContract".to_string()
                ))
            );

            // = when smart_contract exists
            az_smart_contract_hub
                .create(accounts.alice, MOCK_URL.to_string(), 0)
                .unwrap();
            // == when called by non-caller
            set_caller::<DefaultEnvironment>(accounts.charlie);
            // == * it raises an error
            result = az_smart_contract_hub.toggle_enabled(0, false);
            assert_eq!(result, Err(AZSmartContractHubError::Unauthorised));
            // == when called by caller
            set_caller::<DefaultEnvironment>(accounts.bob);
            // === when smart_contract is already enabled
            // ==== when the user tries to enable
            // ==== * it raises an error
            result = az_smart_contract_hub.toggle_enabled(0, true);
            assert_eq!(
                result,
                Err(AZSmartContractHubError::Unchanged("Enabled".to_string()))
            );
            // ==== when the user tries to disable
            // ==== * it updates the smart_contract enabled to false
            result = az_smart_contract_hub.toggle_enabled(0, false);
            assert_eq!(result.unwrap().enabled, false);

            // === when smart_contract is already disabled
            // ==== when the user tries to disable
            // ==== * it raises an error
            result = az_smart_contract_hub.toggle_enabled(0, false);
            assert_eq!(
                result,
                Err(AZSmartContractHubError::Unchanged("Enabled".to_string()))
            );
            // ==== when the user tries to enable
            // ==== * it updates the smart_contract enabled to true
            result = az_smart_contract_hub.toggle_enabled(0, true);
            assert_eq!(result.unwrap().enabled, true);
        }
    }
}
