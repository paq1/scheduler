use async_trait::async_trait;

use crate::models::errors::custom::CustomError;
use crate::models::views::job_view::JobView;

#[async_trait]
pub trait SchedulerApiService {
    async fn get_all_jobs(&self) -> Result<Vec<JobView>, CustomError>;
    async fn get_pending_jobs(&self) -> Result<Vec<JobView>, CustomError>;
    async fn running_one_job(&self, id: &str) -> Result<(), CustomError>;
}