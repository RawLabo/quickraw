erreport::gen_report_code!();

pub(crate) mod tool;
pub mod data;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("target is none")]
    IsNone,
    #[error("{0}")]
    Custom(&'static str)
}