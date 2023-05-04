use std::sync::Arc;
use std::time::Duration;
use clokwerk::{AsyncScheduler, TimeUnits};
use tokio::sync::Mutex;
use tokio::time::sleep;
use crate::app::services::env_service_impl::EnvServiceImpl;
use crate::app::services::scheduler_api_service_impl::SchedulerApiServiceImpl;
use crate::core::services::scheduler_api_service::SchedulerApiService;

#[macro_use]
extern crate lazy_static;

mod app;
mod core;
mod models;

lazy_static! {
    static ref SCHEDULER_API_SERVICE: SchedulerApiServiceImpl = SchedulerApiServiceImpl { env_service: Box::new(EnvServiceImpl::new()) };
}

#[tokio::main]
async fn main() {
    let scheduler = Arc::new(Mutex::new(AsyncScheduler::new()));
    let running = Arc::new(
        Mutex::new(
            true
        )
    );

    tokio::spawn({
        let scheduler_cloned = Arc::clone(&scheduler);
        let running_cloned = Arc::clone(&running);


        async move {
            while *running_cloned.lock().await {

                let pending_jobs = SCHEDULER_API_SERVICE
                    .get_pending_jobs()
                    .await
                    .unwrap_or(vec![]);

                if !pending_jobs.is_empty() {
                    println!("ajout d'un nouveau job {}", pending_jobs.len());
                };

                for job in pending_jobs.into_iter() {

                    let mut scheduler_guard = scheduler_cloned
                        .lock()
                        .await;

                    let job_id = job.id.to_string();
                    let route = job.url;
                    let methode = job.http_method;

                    SCHEDULER_API_SERVICE
                        .running_one_job(job_id.as_str())
                        .await
                        .expect(format!("erreur lors du running du job {}", job_id).as_str());


                    scheduler_guard
                        .every(job.repetition_seconds.unwrap_or(0).seconds())
                        .run(move || {
                            let job_id_cloned = job_id.clone();
                            let route_cloned = route.clone();
                            let methode_cloned = methode.clone();
                            async move {
                                let now = chrono::offset::Utc::now();
                                println!("running : {} at {}", job_id_cloned, now.clone());
                                if methode_cloned.to_uppercase().as_str() == "GET" {
                                    reqwest::get(route_cloned.clone())
                                        .await
                                        // .expect("erreur")
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
        }
    });

    tokio::spawn({
        async move {
            println!("yo");
        }
    });

    loop {
        scheduler.lock().await.run_pending().await;
        sleep(Duration::from_millis(100)).await;
    }
}