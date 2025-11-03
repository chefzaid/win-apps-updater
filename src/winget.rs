use crate::models::UpdatableApp;
use std::process::Command;

/// Get list of updatable applications from winget
pub fn get_updatable_apps() -> Result<Vec<UpdatableApp>, String> {
    let output = Command::new("winget")
        .args(["upgrade", "--include-unknown"])
        .output()
        .map_err(|e| format!("Failed to execute winget: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "winget command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_winget_output(&stdout)
}

/// Parse winget upgrade output
fn parse_winget_output(output: &str) -> Result<Vec<UpdatableApp>, String> {
    let mut apps = Vec::new();
    let lines: Vec<&str> = output.lines().collect();

    // Find the separator line (contains dashes)
    let mut separator_idx = None;

    for (idx, line) in lines.iter().enumerate() {
        if line.contains("---") && line.len() > 20 {
            separator_idx = Some(idx);
            break;
        }
    }

    if separator_idx.is_none() {
        return Ok(apps); // No updates available
    }

    let separator_idx = separator_idx.unwrap();

    // Parse data lines (skip separator)
    for line in lines.iter().skip(separator_idx + 1) {
        let trimmed = line.trim();

        // Stop at empty lines or footer text
        if trimmed.is_empty() || trimmed.contains("upgrades available") {
            break;
        }

        // Parse the line - winget output is space-separated with variable spacing
        let parts: Vec<&str> = trimmed.split_whitespace().collect();

        // We need at least 5 parts: Name, Id, Version, Available, Source
        if parts.len() >= 5 {
            // The last part is the source (winget/msstore)
            let source = parts[parts.len() - 1].to_string();

            // Second to last is Available version
            let available = parts[parts.len() - 2].to_string();

            // Third to last is current Version
            let version = parts[parts.len() - 3].to_string();

            // Fourth to last is the ID
            let id = parts[parts.len() - 4].to_string();

            // Everything before that is the Name
            let name_parts = &parts[0..parts.len() - 4];
            let name = name_parts.join(" ");

            if !name.is_empty() && !id.is_empty() {
                apps.push(UpdatableApp {
                    name,
                    id,
                    version,
                    available,
                    source,
                });
            }
        }
    }

    Ok(apps)
}

/// Update a single application by ID
pub fn update_single_app(app_id: &str) -> Result<String, String> {
    let output = Command::new("winget")
        .args([
            "upgrade",
            "--id",
            app_id,
            "--accept-source-agreements",
            "--accept-package-agreements",
            "-h", // Use -h for silent/headless mode (more compatible than --silent)
        ])
        .output()
        .map_err(|e| format!("Failed to execute winget for {}: {}", app_id, e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check if app needs to be closed
    let combined_output = format!("{}\n{}", stdout, stderr);
    let needs_close = combined_output.contains("application must be closed")
        || combined_output.contains("Close the application")
        || combined_output.contains("currently in use")
        || combined_output.contains("close all instances");

    // winget returns 0 for success, but we should check the output too
    if needs_close {
        Ok(format!("[!] {} - needs to be closed before updating", app_id))
    } else if output.status.success() || output.status.code() == Some(0) {
        // Check if the output indicates success
        if stdout.contains("Successfully installed") || stdout.contains("successfully") {
            Ok(format!("SUCCESS:{} - updated successfully", app_id))
        } else if stdout.contains("No applicable update found") || stdout.contains("No newer package versions") {
            Ok(format!("[i] {} - already up to date", app_id))
        } else if stdout.contains("No package found") {
            Err(format!("FAILURE:{} - package not found", app_id))
        } else {
            Ok(format!("SUCCESS:{} - completed", app_id))
        }
    } else {
        // Extract meaningful error message
        let error_msg = if !stderr.is_empty() {
            stderr.lines()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("Unknown error")
                .trim()
        } else if !stdout.is_empty() {
            stdout
                .lines()
                .filter(|l| !l.trim().is_empty())
                .next_back()
                .unwrap_or("Unknown error")
                .trim()
        } else {
            "Update failed"
        };

        let error_msg = if error_msg.len() > 100 {
            &error_msg[..100]
        } else {
            error_msg
        };

        Err(format!("FAILURE:{} - {}", app_id, error_msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_output() {
        let output = "";
        let result = parse_winget_output(output);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_parse_sample_output() {
        let output = r#"
Name                           Id                          Version        Available      Source
-------------------------------------------------------------------------------------------------
Microsoft Visual Studio Code   Microsoft.VisualStudioCode  1.85.0         1.85.1         winget
Google Chrome                  Google.Chrome               120.0.6099.109 120.0.6099.130 winget
2 upgrades available.
"#;
        let result = parse_winget_output(output);
        assert!(result.is_ok());
        let apps = result.unwrap();
        assert_eq!(apps.len(), 2);
    }
}

