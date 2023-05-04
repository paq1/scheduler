use std::sync::Arc;
use std::sync::mpsc::channel;
use std::time::Duration;
use clokwerk::AsyncScheduler;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tokio::time::sleep;
use crate::app::services::env_service_impl::EnvServiceImpl;
use crate::app::services::scheduler_api_service_impl::SchedulerApiServiceImpl;
use crate::core::helpers::scheduler_helper::SchedulerHelper;
use crate::core::services::scheduler_api_service::SchedulerApiService;

mod app;
mod core;
mod models;

static SCHEDULER_API_SERVICE: Lazy<SchedulerApiServiceImpl> = Lazy::new(|| {
    SchedulerApiServiceImpl { env_service: Box::new(EnvServiceImpl::new()) }
});

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
                    scheduler_cloned
                        .lock().await
                        .add_job(job).await;
                }
            }
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
            // println!("ending scheduler");
        }
    });

    ctrlc::set_handler(move || {
        tx.send(()).expect("couldn't send signals");
    }).expect("Error setting ctrl-c handler");
    rx.recv().expect("couldn't receive from channel.");
    synchroniz_api_thread.abort();
    update_scheduler_thread.abort();
    *running.lock().await = false;

    SCHEDULER_API_SERVICE
        .pending_all()
        .await
        .expect("erreur lors du pending des jobs");

    println!("closed");
}
