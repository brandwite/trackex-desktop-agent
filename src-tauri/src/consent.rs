// Consent logic is primarily handled in the storage::consent module

#[allow(dead_code)]
pub async fn is_consent_required() -> bool {
    match crate::storage::consent::get_consent_status().await {
        Ok(status) => !status.accepted,
        Err(_) => true, // Require consent if we can't determine status
    }
}