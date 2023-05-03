use crate::models::errors::custom::CustomError;

pub trait EnvService: 'static + Sync + Send {
    fn get_url_api(&self) -> Result<String, CustomError>;
}