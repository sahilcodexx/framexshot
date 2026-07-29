//! OCR module — Linux implementation using tesseract CLI

use crate::utils::AppResult;
use std::process::Command;

/// Recognize text from an image using tesseract OCR.
/// Requires `tesseract` to be installed: apt install tesseract-ocr
pub fn recognize_text_from_image(image_path: &str) -> AppResult<String> {
    let output = Command::new("tesseract")
        .arg(image_path)
        .arg("stdout")
        .arg("--psm")
        .arg("3") // fully automatic page segmentation
        .output()
        .map_err(|e| {
            format!(
                "Failed to run tesseract: {}. Please install it with: sudo apt install tesseract-ocr",
                e
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tesseract failed: {}", stderr));
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if text.is_empty() {
        return Err("No text recognized in image".to_string());
    }

    Ok(text)
}
