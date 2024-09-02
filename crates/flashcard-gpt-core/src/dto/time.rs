use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Time {
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>
}
