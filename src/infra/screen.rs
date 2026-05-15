//! Windows screen capture helpers.

use anyhow::{Result, anyhow};

#[derive(Clone, Copy, Debug)]
pub struct ScreenBounds {
    pub left: i32,
    pub top: i32,
}

#[derive(Debug)]
pub struct ScreenImage {
    pub bounds: ScreenBounds,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Captures the primary monitor into an RGBA image buffer.
#[cfg(target_os = "windows")]
pub fn capture_primary_screen_image() -> Result<ScreenImage> {
    let monitor = xcap::Monitor::all()?
        .into_iter()
        .find(|monitor| monitor.is_primary())
        .ok_or_else(|| anyhow!("Could not get the primary monitor."))?;
    let image = monitor.capture_image()?;
    let width = image.width();
    let height = image.height();

    Ok(ScreenImage {
        bounds: ScreenBounds {
            left: monitor.x(),
            top: monitor.y(),
        },
        width,
        height,
        rgba: image.into_raw(),
    })
}

/// Captures the primary monitor into an RGBA image buffer.
#[cfg(not(target_os = "windows"))]
pub fn capture_primary_screen_image() -> Result<ScreenImage> {
    Err(anyhow!(
        "Native screen capture is currently implemented for Windows only."
    ))
}
