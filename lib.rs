#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod az_smart_contract_metadata_hub {
    use ink::storage::Mapping;

    // === ENUMS ===
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum AzSmartContractMetadataHubError {
        AlreadyLiked,
        NotFound(String),
    }

    // === EVENTS (To be used with Subsquid) ===

    // === STRUCTS ===
    #[derive(scale::Decode, scale::Encode, Debug, Clone, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Record {
        id: u32,
        smart_contract_address: AccountId,
        likes: u16,
        dislikes: u16,
        submitter: AccountId,
    }

    #[derive(Debug, Default)]
    #[ink::storage_item]
    pub struct Records {
        values: Mapping<u32, Record>,
        length: u32,
    }
    impl Records {
        //     pub fn index(&self, page: u32, size: u16) -> Vec<Record> {
        //         let mut records: Vec<Record> = vec![];
        //         // When there's no records
        //         if self.length == 0 {
        //             return records;
        //         }

        //         let records_to_skip: Option<u32> = if page == 0 {
        //             Some(0)
        //         } else {
        //             page.checked_mul(size.into())
        //         };
        //         let starting_index: u32;
        //         let ending_index: u32;
        //         // When the records to skip is greater than max possible
        //         if let Some(records_to_skip_unwrapped) = records_to_skip {
        //             let ending_index_wrapped: Option<u32> =
        //                 self.length.checked_sub(records_to_skip_unwrapped);
        //             // When records to skip is greater than total number of records
        //             if ending_index_wrapped.is_none() {
        //                 return records;
        //             }
        //             ending_index = ending_index_wrapped.unwrap();
        //             starting_index = ending_index.saturating_sub(size.into());
        //         } else {
        //             return records;
        //         }
        //         for i in (starting_index..=ending_index).rev() {
        //             records.push(self.values.get(i).unwrap())
        //         }
        //         records
        //     }

        pub fn create(
            &mut self,
            smart_contract_address: AccountId,
            submitter: AccountId,
        ) -> Result<Record, AzSmartContractMetadataHubError> {
            let record: Record = Record {
                id: self.length,
                smart_contract_address,
                likes: 1,
                dislikes: 0,
                submitter,
            };
            self.values.insert(self.length, &record);
            self.length += 1;

            Ok(record)
        }

        pub fn update(&mut self, value: &Record) {
            self.values.insert(value.id, value);
        }
    }

    // === CONTRACT ===
    #[ink(storage)]
    #[derive(Default)]
    pub struct AzSmartContractMetadataHub {
        records: Records,
        user_ratings: Mapping<(u32, AccountId), i8>,
    }
    impl AzSmartContractMetadataHub {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                records: Records {
                    values: Mapping::default(),
                    length: 0,
                },
                user_ratings: Mapping::default(),
            }
        }

        // === QUERIES ===
        #[ink(message)]
        pub fn show(&self, id: u32) -> Result<Record, AzSmartContractMetadataHubError> {
            if let Some(record) = self.records.values.get(id) {
                Ok(record)
            } else {
                Err(AzSmartContractMetadataHubError::NotFound(
                    "Record".to_string(),
                ))
            }
        }

        // === HANDLES ===
        #[ink(message)]
        pub fn create(
            &mut self,
            smart_contract_address: AccountId,
        ) -> Result<Record, AzSmartContractMetadataHubError> {
            let caller: AccountId = Self::env().caller();
            let record: Record = self.records.create(smart_contract_address, caller)?;
            self.user_ratings.insert((record.id, caller), &1);

            Ok(record)
        }

        #[ink(message)]
        pub fn like(&mut self, id: u32) -> Result<Record, AzSmartContractMetadataHubError> {
            if let Some(mut record) = self.records.values.get(id) {
                let caller: AccountId = Self::env().caller();
                // Get current user rating or create
                if let Some(rating) = self.user_ratings.get((id, caller)) {
                    if rating == 1 {
                        return Err(AzSmartContractMetadataHubError::AlreadyLiked);
                    } else if rating == -1 {
                        record.dislikes -= 1
                    }
                }
                record.likes += 1;
                self.records.update(&record);
                self.user_ratings.insert((id, caller), &1);

                Ok(record)
            } else {
                Err(AzSmartContractMetadataHubError::NotFound(
                    "Record".to_string(),
                ))
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::{
            test::{default_accounts, set_caller, DefaultAccounts},
            DefaultEnvironment,
        };

        // === HELPERS ===
        fn init() -> (
            DefaultAccounts<DefaultEnvironment>,
            AzSmartContractMetadataHub,
        ) {
            let accounts = default_accounts();
            set_caller::<DefaultEnvironment>(accounts.bob);
            let az_smart_contract_metadata_hub = AzSmartContractMetadataHub::new();
            (accounts, az_smart_contract_metadata_hub)
        }

        // === TESTS ===
        // === TEST QUERIES ===
        #[ink::test]
        fn test_show() {
            let (accounts, mut az_smart_contract_metadata_hub) = init();
            // = when record does not exist
            // * it returns error
            assert_eq!(
                az_smart_contract_metadata_hub.show(0),
                Err(AzSmartContractMetadataHubError::NotFound(
                    "Record".to_string()
                ))
            );
            // = when record exists
            let record: Record = az_smart_contract_metadata_hub
                .records
                .create(accounts.alice, accounts.bob)
                .unwrap();
            // = * it returns the record
            assert_eq!(az_smart_contract_metadata_hub.show(record.id), Ok(record));
        }

        // === TEST HANDLES ===
        #[ink::test]
        fn test_create() {
            let (accounts, mut az_smart_contract_metadata_hub) = init();
            az_smart_contract_metadata_hub.records.length = u32::MAX - 1;
            // * it stores the submitter as the caller
            let result = az_smart_contract_metadata_hub.create(accounts.alice);
            let result_unwrapped = result.unwrap();
            // * it stores the id as the current length
            assert_eq!(result_unwrapped.id, u32::MAX - 1);
            // * it increases the records length by 1
            assert_eq!(az_smart_contract_metadata_hub.records.length, u32::MAX);
            // * it stores the submitted smart contract address
            assert_eq!(result_unwrapped.smart_contract_address, accounts.alice);
            // * it sets the like to 1 and dislike to 0
            assert_eq!(result_unwrapped.likes, 1);
            assert_eq!(result_unwrapped.dislikes, 0);
            // * it sets the submitter to the caller
            assert_eq!(result_unwrapped.submitter, accounts.bob);
            // * it sets the user_rating to 1
            assert_eq!(
                az_smart_contract_metadata_hub
                    .user_ratings
                    .get((result_unwrapped.id, result_unwrapped.submitter))
                    .unwrap(),
                1
            );
            // * it stores the record
            assert_eq!(
                result_unwrapped,
                az_smart_contract_metadata_hub
                    .records
                    .values
                    .get(result_unwrapped.id)
                    .unwrap()
            );
        }

        #[ink::test]
        fn test_like() {
            let (accounts, mut az_smart_contract_metadata_hub) = init();
            // = when record doesn't exist
            // = * it raises an error
            let mut result = az_smart_contract_metadata_hub.like(0);
            assert_eq!(
                result,
                Err(AzSmartContractMetadataHubError::NotFound(
                    "Record".to_string()
                ))
            );

            // = when record exists
            az_smart_contract_metadata_hub
                .create(accounts.alice)
                .unwrap();
            // == when user has already liked
            // == * it raises an error
            result = az_smart_contract_metadata_hub.like(0);
            assert_eq!(result, Err(AzSmartContractMetadataHubError::AlreadyLiked));
            // == when user has not liked or disliked
            set_caller::<DefaultEnvironment>(accounts.alice);
            // == * it returns the updated record
            let mut record = az_smart_contract_metadata_hub.like(0).unwrap();
            // == * it increases the like count by 1
            assert_eq!(record.likes, 2);
            // == * it does change the dislike count
            assert_eq!(record.dislikes, 0);
            // == * it sets the user rating to 1
            assert_eq!(
                az_smart_contract_metadata_hub
                    .user_ratings
                    .get((0, accounts.alice))
                    .unwrap(),
                1
            );
            // == when user has already disliked
            set_caller::<DefaultEnvironment>(accounts.charlie);
            record.dislikes += 1;
            az_smart_contract_metadata_hub.records.update(&record);
            az_smart_contract_metadata_hub
                .user_ratings
                .insert((0, accounts.charlie), &-1);
            record = az_smart_contract_metadata_hub.like(0).unwrap();
            // == * it increses the like cout by 1
            assert_eq!(record.likes, 3);
            // == * it decreases the dislike count by 1
            assert_eq!(record.dislikes, 0);
            // == * it sets the user rating to 1
            assert_eq!(
                az_smart_contract_metadata_hub
                    .user_ratings
                    .get((0, accounts.charlie))
                    .unwrap(),
                1
            );
        }
    }
}
