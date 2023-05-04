use std::sync::Arc;
use std::sync::mpsc::channel;
use std::time::Duration;
use clokwerk::{AsyncScheduler, TimeUnits};
use tokio::sync::Mutex;
use tokio::time::sleep;
use crate::app::services::env_service_impl::EnvServiceImpl;
use crate::app::services::scheduler_api_service_impl::SchedulerApiServiceImpl;
use crate::core::services::scheduler_api_service::SchedulerApiService;
use crate::models::views::job_view::JobView;

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

    let (tx, rx) = channel();

    let scheduler = Arc::new(Mutex::new(AsyncScheduler::new()));
    let running = Arc::new(
        Mutex::new(
            true
        )
    );

    let synchroniz_api_thread = tokio::spawn({
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

                    add_job(&mut scheduler_guard, job).await;
                }
            }
            println!("ending synchronisation scheduler");
        }
    });

    let update_scheduler_thread = tokio::spawn({
        let arc_scheduler = Arc::clone(&scheduler);
        let arc_running = Arc::clone(&running);
        async move {
            while *arc_running.lock().await {
                arc_scheduler
                    .lock().await
                    .run_pending().await;
                sleep(Duration::from_millis(100)).await;
            }
            println!("ending scheduler");
        }
    });

    ctrlc::set_handler(move || {
        tx.send(()).expect("couldn't send signals");
    }).expect("Error setting ctrl-c handler");
    rx.recv().expect("couldn't receive from channel.");
    synchroniz_api_thread.abort();
    update_scheduler_thread.abort();
    *running.lock().await = false;
    println!("closed");
}

async fn add_job(scheduler: &mut AsyncScheduler, job: JobView) {
    SCHEDULER_API_SERVICE
        .running_one_job(job.id.as_str())
        .await
        .expect(format!("erreur lors du running du job {}", job.id).as_str());


    scheduler
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
