use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScreenshotError {
    #[error("BrowserBuildErr: {0}")]
    BrowserCreateErr(String),
    #[error("TabCreateErr: {0}")]
    TabCreateErr(String),
    #[error("TabOperateErr: {0}")]
    TabOperateErr(String),
    #[error("InvalidFilePath: {0}")]
    InvalidFilePath(String),
    #[error("ScreenshotCreateErr: {0}")]
    ScreenshotCreateErr(String),
}
