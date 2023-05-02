use crate::core::services::env_service::EnvService;
use crate::models::errors::custom::CustomError;

pub struct EnvServiceImpl {}

impl EnvServiceImpl {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        Self {}
    }
}

impl EnvService for EnvServiceImpl {
    fn get_url_api(&self) -> Result<String, CustomError> {
        std::env::var("SCHEDULER_API")
            .map_err(|_| CustomError::new("erreur lors de la lecture des venv"))
    }
}