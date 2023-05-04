use async_trait::async_trait;
use crate::models::views::job_view::JobView;

#[async_trait]
pub trait SchedulerHelper {
    async fn add_job(&mut self, job: JobView);
}