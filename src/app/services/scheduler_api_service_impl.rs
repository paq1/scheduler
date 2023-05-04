use async_trait::async_trait;
use futures::TryFutureExt;
use crate::core::services::env_service::EnvService;
use crate::core::services::scheduler_api_service::SchedulerApiService;
use crate::models::errors::custom::CustomError;
use crate::models::views::job_view::JobView;

pub struct SchedulerApiServiceImpl {
    pub env_service: Box<dyn EnvService>
}

#[async_trait]
impl SchedulerApiService for SchedulerApiServiceImpl {
    async fn get_all_jobs(&self) -> Result<Vec<JobView>, CustomError> {
        let url = self.env_service.get_url_api()?;
        reqwest::get(format!("{}/tasks", url))
            .and_then(|response| {
                response.json::<Vec<JobView>>()
            })
            .await
            .map_err(|_| CustomError::new("erreur lors de la recuperation de jobs"))
    }

    async fn get_pending_jobs(&self) -> Result<Vec<JobView>, CustomError> {
        self
            .get_all_jobs()
            .await
            .map(|jobs| {
                jobs
                    .into_iter()
                    .filter(|job| job.state == "pending")
                    .collect::<Vec<_>>()
            })
    }

    async fn running_one_job(&self, id: &str) -> Result<(), CustomError> {
        let url = self.env_service.get_url_api()?;
        reqwest::Client::new()
            .put(format!("{}/tasks/commands/running/{}", url, id))
            .send()
            .await
            .map(|_| ())
            .map_err(|_| CustomError::new("erreur lors du changement d'etat"))
    }

    async fn running_jobs(&self, jobs: Vec<JobView>) -> Result<(), CustomError> {

        for job in jobs.iter() {
            self
                .running_one_job(job.id.as_str())
                .await
                .expect("erreur lors de l'envoi");
        };

        Ok(())
    }

    async fn pending_all(&self) -> Result<(), CustomError> {
        let url = self.env_service.get_url_api()?;
        reqwest::Client::new()
            .put(format!("{}/tasks/commands/pending_all", url))
            .send()
            .await
            .map(|_| ())
            .map_err(|_| CustomError::new("erreur lors du changement d'etat"))
    }
}