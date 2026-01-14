use log::{error, info};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub enum ConversionFormat {
    Mp4,
    Mov,
    Mp3,
    Webm,
}

impl ConversionFormat {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "mp4" => ConversionFormat::Mp4,
            "mov" => ConversionFormat::Mov,
            "mp3" => ConversionFormat::Mp3,
            "webm" => ConversionFormat::Webm,
            _ => ConversionFormat::Mp4, // default
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            ConversionFormat::Mp4 => "mp4",
            ConversionFormat::Mov => "mov",
            ConversionFormat::Mp3 => "mp3",
            ConversionFormat::Webm => "webm",
        }
    }
}

pub fn convert_video(
    input_path: &Path,
    output_path: &Path,
    format: ConversionFormat,
) -> Result<(), String> {
    info!(
        "Converting {:?} to {:?} (format: {:?})",
        input_path, output_path, format
    );

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-i").arg(input_path);

    // Add format-specific arguments
    match format {
        ConversionFormat::Mp4 => {
            // High quality MP4 with H.264 codec
            cmd.arg("-c:v")
                .arg("libx264")
                .arg("-preset")
                .arg("medium")
                .arg("-crf")
                .arg("23")
                .arg("-c:a")
                .arg("aac")
                .arg("-b:a")
                .arg("192k")
                .arg("-movflags")
                .arg("+faststart");
        }
        ConversionFormat::Mov => {
            // MOV format with H.264
            cmd.arg("-c:v")
                .arg("libx264")
                .arg("-preset")
                .arg("medium")
                .arg("-crf")
                .arg("23")
                .arg("-c:a")
                .arg("aac")
                .arg("-b:a")
                .arg("192k");
        }
        ConversionFormat::Mp3 => {
            // Extract audio as MP3
            cmd.arg("-vn") // no video
                .arg("-c:a")
                .arg("libmp3lame")
                .arg("-q:a")
                .arg("2") // high quality
                .arg("-b:a")
                .arg("192k");
        }
        ConversionFormat::Webm => {
            // WebM format
            cmd.arg("-c:v")
                .arg("libvpx-vp9")
                .arg("-crf")
                .arg("30")
                .arg("-b:v")
                .arg("0")
                .arg("-c:a")
                .arg("libopus")
                .arg("-b:a")
                .arg("128k");
        }
    }

    // Overwrite output file if exists
    cmd.arg("-y");
    cmd.arg(output_path);

    // Execute FFmpeg
    let output = cmd.output().map_err(|e| {
        let err_msg = format!("Failed to execute FFmpeg: {}", e);
        error!("{}", err_msg);
        err_msg
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let err_msg = format!("FFmpeg conversion failed: {}", stderr);
        error!("{}", err_msg);
        return Err(err_msg);
    }

    info!("Conversion completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_str() {
        assert!(matches!(ConversionFormat::from_str("mp4"), ConversionFormat::Mp4));
        assert!(matches!(ConversionFormat::from_str("MP4"), ConversionFormat::Mp4));
        assert!(matches!(ConversionFormat::from_str("mov"), ConversionFormat::Mov));
        assert!(matches!(ConversionFormat::from_str("mp3"), ConversionFormat::Mp3));
    }

    #[test]
    fn test_extension() {
        assert_eq!(ConversionFormat::Mp4.extension(), "mp4");
        assert_eq!(ConversionFormat::Mov.extension(), "mov");
        assert_eq!(ConversionFormat::Mp3.extension(), "mp3");
    }
}
