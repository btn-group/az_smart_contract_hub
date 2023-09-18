use ink::{
    env::Error as InkEnvError,
    prelude::{format, string::String},
    LangError,
};
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum AZSmartContractHubError {
    ContractCall(LangError),
    InkEnvError(String),
    NotFound(String),
    Unauthorised,
    Unchanged(String),
    AZGroupsError(AZGroupsError),
}
impl From<AZGroupsError> for AZSmartContractHubError {
    fn from(error: AZGroupsError) -> Self {
        AZSmartContractHubError::AZGroupsError(error)
    }
}
impl From<InkEnvError> for AZSmartContractHubError {
    fn from(e: InkEnvError) -> Self {
        AZSmartContractHubError::InkEnvError(format!("{e:?}"))
    }
}
impl From<LangError> for AZSmartContractHubError {
    fn from(e: LangError) -> Self {
        AZSmartContractHubError::ContractCall(e)
    }
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum AZGroupsError {
    ContractCall(LangError),
    InkEnvError(String),
    NotFound(String),
    Unauthorised,
    UnprocessableEntity(String),
}
