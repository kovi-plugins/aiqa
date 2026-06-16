use std::path::Path;

use headless_chrome::Browser;
use headless_chrome::protocol::cdp::{Emulation, Page};
use headless_chrome::types::Bounds;

use crate::error::ScreenshotError;

pub struct ScreenshotManager {
    browser: Browser,
}

impl ScreenshotManager {
    pub fn init() -> Result<Self, ScreenshotError> {
        let browser =
            Browser::default().map_err(|err| ScreenshotError::BrowserCreateErr(err.to_string()))?;

        Ok(Self { browser })
    }

    pub fn screenshot<P: AsRef<Path>>(
        &mut self,
        full_file_path: P,
    ) -> Result<Vec<u8>, ScreenshotError> {
        let file_path = full_file_path.as_ref();

        let tab = match self.browser.new_tab() {
            Ok(tab) => tab,
            Err(_) => {
                self.restart_browser().map_err(|restart_err| {
                    ScreenshotError::TabCreateErr(restart_err.to_string())
                })?;
                self.browser
                    .new_tab()
                    .map_err(|new_tab_err| ScreenshotError::TabCreateErr(new_tab_err.to_string()))?
            }
        };

        tab.navigate_to(&format!(
            "file://{}",
            file_path
                .to_str()
                .ok_or(ScreenshotError::InvalidFilePath("".to_string()))?
        ))
        .map_err(|err| ScreenshotError::InvalidFilePath(err.to_string()))?;

        tab.wait_for_element("div.finish")
            .map_err(|err| ScreenshotError::TabOperateErr(err.to_string()))?;

        let viewport = tab
            .wait_for_element("article.markdown-body")
            .map_err(|err| ScreenshotError::TabOperateErr(err.to_string()))?
            .get_box_model()
            .map_err(|err| ScreenshotError::TabOperateErr(err.to_string()))?
            .margin_viewport();

        // println!("111111111: {:?}", viewport);

        tab.set_bounds(Bounds::Normal {
            left: Some(0),
            top: Some(0),
            width: Some(viewport.width),
            height: Some(viewport.height + 200.0),
        })
        .map_err(|err| ScreenshotError::TabOperateErr(err.to_string()))?;

        // 设置设备像素比
        tab.call_method(Emulation::SetDeviceMetricsOverride {
            width: viewport.width as u32,
            height: (viewport.height + 200.0) as u32,
            device_scale_factor: 2.0,
            mobile: false,
            scale: None,
            screen_width: None,
            screen_height: None,
            position_x: None,
            position_y: None,
            dont_set_visible_size: None,
            screen_orientation: None,
            viewport: None,
            display_feature: None,
            device_posture: None,
        })
        .map_err(|err| ScreenshotError::TabOperateErr(err.to_string()))?;

        let png_data = tab
            .capture_screenshot(
                Page::CaptureScreenshotFormatOption::Png,
                None,
                Some(viewport),
                true,
            )
            .map_err(|err| ScreenshotError::ScreenshotCreateErr(err.to_string()))?;

        Ok(png_data)
    }

    fn restart_browser(&mut self) -> Result<(), ScreenshotError> {
        let browser =
            Browser::default().map_err(|err| ScreenshotError::BrowserCreateErr(err.to_string()))?;
        self.browser = browser;

        Ok(())
    }
}
