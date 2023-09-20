use crate::errors::AZSmartContractHubError;

pub fn validate_presence_of(string: &str, field_name: &str) -> Result<(), AZSmartContractHubError> {
    if string.is_empty() {
        return Err(AZSmartContractHubError::UnprocessableEntity(format!(
            "{field_name} can't be blank"
        )));
    };

    Ok(())
}
