use async_trait::async_trait;
use clokwerk::{AsyncScheduler, TimeUnits};
use crate::core::helpers::scheduler_helper::SchedulerHelper;
use crate::core::services::scheduler_api_service::SchedulerApiService;
use crate::models::views::job_view::JobView;
use crate::SCHEDULER_API_SERVICE;

#[async_trait]
impl SchedulerHelper for AsyncScheduler {
    async fn add_job(&mut self, job: JobView) {
        SCHEDULER_API_SERVICE
            .running_one_job(job.id.as_str())
            .await
            .expect(format!("erreur lors du running du job {}", job.id).as_str());


        self
            .every(job.repetition_seconds.unwrap_or(0).seconds())
            .run(move || {
                let job_id_cloned = job.id.clone();
                let route_cloned = job.url.clone();
                let methode_cloned = job.http_method.clone();
                async move {
                    let now = chrono::offset::Utc::now();
                    println!("running : {} at {}", job_id_cloned, now.clone());
                    if methode_cloned.to_uppercase().as_str() == "GET" {
                        reqwest::get(route_cloned.clone())
                            .await
                            .map(|response| {
                                if response.status().as_u16() == 200u16 {
                                    println!("called : {} at {:?}", route_cloned, now);
                                };
                            })
                            .expect(format!("erreur lors de l'execution de la requete du job {}", job_id_cloned).as_str());
                    };
                }
            });
    }
}