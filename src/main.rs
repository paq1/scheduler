use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use tokio::time::Duration;
use clokwerk::{AsyncScheduler, TimeUnits};
use crate::app::services::env_service_impl::EnvServiceImpl;
use crate::app::services::scheduler_api_service_impl::SchedulerApiServiceImpl;
use crate::core::services::scheduler_api_service::SchedulerApiService;

mod app;
mod core;
mod models;

#[tokio::main]
async fn main() {
    println!("lancement du scheduler");
    let (tx, rx) = channel();

    let scheduler = Arc::new(
        Mutex::new(AsyncScheduler::new())
    );

    let running = Arc::new(
        Mutex::new(
            true
        )
    );

    let thread_scheduler = tokio::spawn({
        let scheduler_cloned = Arc::clone(&scheduler);
        let running_cloned = Arc::clone(&running);

        let env_service = EnvServiceImpl::new();
        let scheduler_api_service = SchedulerApiServiceImpl { env_service: Box::new(env_service)};

        async move {

            while *running_cloned.lock().unwrap() {

                let pending_jobs = scheduler_api_service
                    .get_pending_jobs()
                    .await
                    .unwrap_or(vec![]);

                println!("nombre de jobs : {}", pending_jobs.len());


                for job in pending_jobs.into_iter() {

                    let mut scheduler_guard = scheduler_cloned
                        .lock()
                        .unwrap();

                    let job_id = job.id.to_string();

                    scheduler_guard.every(job.repetition_seconds.unwrap_or(0).seconds()).run(move || {
                        let job_id_cloned = job_id.to_string();
                        async move {
                            println!("{}", job_id_cloned);
                        }
                    });
                }
                println!("hello");
                thread::sleep(Duration::from_secs(2));
            }


        }
    });

    ctrlc::set_handler(move || {
        tx.send(()).expect("couldn't send signals");
    }).expect("Error setting ctrl-c handler");
    rx.recv().expect("couldn't receive from channel.");
    thread_scheduler.abort();

    println!("ici");
    let running_cloned = Arc::clone(&running);
    let mut running_guard = running_cloned.lock().unwrap();
    *running_guard = false;
}
