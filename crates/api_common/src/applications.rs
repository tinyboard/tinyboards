use serde::{Deserialize, Serialize};
use tinyboards_db_views::structs::RegistrationApplicationView;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListRegistrationApplications {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListRegistrationApplicationsResponse {
    pub applications: Vec<RegistrationApplicationView>,
    pub total_count: i64,
}