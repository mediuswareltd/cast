use log::warn;
use std::path::Path;

/// Validate that the conversion from input format to output format is supported
pub fn validate_conversion(input_ext: &str, output_format: &str) -> Result<(), String> {
    let input = input_ext.to_lowercase();
    let output = output_format.to_lowercase();

    // Define supported conversions
    let valid_conversions = vec![
        ("mov", "mp4"),
        ("webm", "mp4"),
        ("mp4", "mov"),
        ("mp4", "mp3"),
    ];

    for (from, to) in &valid_conversions {
        if input == *from && output == *to {
            return Ok(());
        }
    }

    let error_msg = format!(
        "Unsupported conversion: {} to {}. Supported conversions: MOV→MP4, WEBM→MP4, MP4→MOV, MP4→MP3",
        input.to_uppercase(),
        output.to_uppercase()
    );

    warn!("{}", error_msg);
    Err(error_msg)
}

/// Validate file size
pub fn validate_file_size(file_path: &Path, max_size: usize) -> Result<(), String> {
    let metadata = std::fs::metadata(file_path).map_err(|e| {
        format!("Failed to read file metadata: {}", e)
    })?;

    let file_size = metadata.len() as usize;

    if file_size > max_size {
        let error_msg = format!(
            "File size ({:.2} MB) exceeds maximum allowed size ({} MB)",
            file_size as f64 / (1024.0 * 1024.0),
            max_size / (1024 * 1024)
        );
        warn!("{}", error_msg);
        return Err(error_msg);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_conversions() {
        assert!(validate_conversion("mov", "mp4").is_ok());
        assert!(validate_conversion("MOV", "MP4").is_ok());
        assert!(validate_conversion("webm", "mp4").is_ok());
        assert!(validate_conversion("mp4", "mov").is_ok());
        assert!(validate_conversion("mp4", "mp3").is_ok());
    }

    #[test]
    fn test_invalid_conversions() {
        assert!(validate_conversion("mov", "mp3").is_err());
        assert!(validate_conversion("avi", "mp4").is_err());
        assert!(validate_conversion("mp4", "webm").is_err());
    }
}
