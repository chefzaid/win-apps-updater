use crate::models::UpdatableApp;
use std::process::Command;

/// Retrieves the list of updatable applications from winget.
pub fn get_updatable_apps() -> Result<Vec<UpdatableApp>, String> {
    let output = Command::new("winget")
        .args(["upgrade", "--include-unknown"])
        .output()
        .map_err(|e| format!("Failed to execute winget: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "winget command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_winget_output(&stdout)
}

/// Column positions parsed from the winget header line.
struct ColumnLayout {
    id_col: usize,
    version_col: usize,
    available_col: usize,
    source_col: usize,
}

/// Sanitizes raw winget output by resolving carriage returns.
///
/// Winget prints progress spinners using `\r` to overwrite the current line.
/// When captured via `Command::output()`, these `\r` characters remain inline.
/// For each line, we keep only the text after the last `\r`, which is the
/// final visible content (mimicking terminal behaviour).
fn sanitize_output(output: &str) -> String {
    output
        .lines()
        .map(|line| {
            // The last \r-delimited segment is what the terminal would display.
            match line.rfind('\r') {
                Some(pos) => &line[pos + 1..],
                None => line,
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Detects whether winget output has been line-wrapped at a fixed console width
/// and reconstructs the original wide lines.
///
/// Winget wraps its table output when stdout is piped to a narrow console buffer
/// (commonly 80 or 120 columns). The separator line of dashes is the most
/// reliable indicator: if it spans multiple consecutive all-dash lines, the
/// first line's length gives us the wrap width.
fn unwrap_output(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();

    let wrap_width = detect_wrap_width(&lines);
    let Some(width) = wrap_width else {
        return output.to_string();
    };

    let mut result: Vec<String> = Vec::new();
    let mut current = String::new();

    for line in &lines {
        current.push_str(line);
        if line.len() < width {
            result.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        result.push(current);
    }

    result.join("\n")
}

/// Returns the wrap width if the output appears to be line-wrapped.
///
/// Detection: look for two (or more) consecutive lines consisting entirely of
/// dashes. The first such line's length is the wrap width.
fn detect_wrap_width(lines: &[&str]) -> Option<usize> {
    for pair in lines.windows(2) {
        let a = pair[0];
        let b = pair[1];
        if !a.is_empty()
            && !b.is_empty()
            && a.chars().all(|c| c == '-')
            && b.chars().all(|c| c == '-')
        {
            return Some(a.len());
        }
    }
    None
}

/// Parses the winget upgrade output into a list of [`UpdatableApp`]s.
///
/// First sanitizes the output (stripping carriage-return progress indicators and
/// unwrapping line-wrapped output), then uses column positions from the header
/// line so that app names containing spaces are parsed correctly.
pub fn parse_winget_output(output: &str) -> Result<Vec<UpdatableApp>, String> {
    let sanitized = sanitize_output(output);
    let unwrapped = unwrap_output(&sanitized);
    let lines: Vec<&str> = unwrapped.lines().collect();

    // Locate the header line (e.g. "Name   Id   Version   Available   Source").
    let header_idx = lines.iter().position(|line| {
        line.contains("Name") && line.contains("Id") && line.contains("Version")
    });

    let header_idx = match header_idx {
        Some(idx) => idx,
        None => return Ok(Vec::new()), // No updates available
    };

    let header = lines[header_idx];
    let layout = ColumnLayout {
        id_col: header
            .find("Id")
            .ok_or("Missing Id column in winget output")?,
        version_col: header
            .find("Version")
            .ok_or("Missing Version column in winget output")?,
        available_col: header
            .find("Available")
            .ok_or("Missing Available column in winget output")?,
        source_col: header
            .find("Source")
            .ok_or("Missing Source column in winget output")?,
    };

    // Find the separator line that follows the header.
    let data_start = lines
        .iter()
        .enumerate()
        .skip(header_idx + 1)
        .find(|(_, line)| line.contains("---") && line.len() > 20)
        .map(|(i, _)| i + 1)
        .unwrap_or(header_idx + 1);

    let mut apps = Vec::new();
    for line in lines.iter().skip(data_start) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.contains("upgrades available") {
            break;
        }
        if let Some(app) = parse_app_line(line, &layout) {
            apps.push(app);
        }
    }

    Ok(apps)
}

/// Parses a single data line using the known column layout.
fn parse_app_line(line: &str, layout: &ColumnLayout) -> Option<UpdatableApp> {
    if line.len() <= layout.id_col {
        return None;
    }

    let name = safe_slice(line, 0, layout.id_col).trim().to_string();
    let id = safe_slice(line, layout.id_col, layout.version_col)
        .trim()
        .to_string();
    let version = safe_slice(line, layout.version_col, layout.available_col)
        .trim()
        .to_string();
    let available = safe_slice(line, layout.available_col, layout.source_col)
        .trim()
        .to_string();
    let source = if line.len() > layout.source_col {
        line[layout.source_col..].trim().to_string()
    } else {
        String::new()
    };

    if name.is_empty() || id.is_empty() {
        return None;
    }

    Some(UpdatableApp::new(name, id, version, available, source))
}

/// Safely slices a string by byte range, clamping to the string length.
fn safe_slice(s: &str, start: usize, end: usize) -> &str {
    let start = start.min(s.len());
    let end = end.min(s.len());
    &s[start..end]
}

/// Updates a single application by its winget ID.
pub fn update_single_app(app_id: &str) -> Result<String, String> {
    let output = Command::new("winget")
        .args([
            "upgrade",
            "--id",
            app_id,
            "--accept-source-agreements",
            "--accept-package-agreements",
            "-h",
        ])
        .output()
        .map_err(|e| format!("Failed to execute winget for {app_id}: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    classify_update_result(app_id, output.status.success(), &stdout, &combined)
}

/// Classifies the update result based on winget output.
fn classify_update_result(
    app_id: &str,
    success: bool,
    stdout: &str,
    combined: &str,
) -> Result<String, String> {
    let needs_close = combined.contains("application must be closed")
        || combined.contains("Close the application")
        || combined.contains("currently in use")
        || combined.contains("close all instances");

    if needs_close {
        return Ok(format!(
            "[!] {app_id} - needs to be closed before updating"
        ));
    }

    if success {
        return Ok(classify_success(app_id, stdout));
    }

    Err(format!(
        "FAILURE:{app_id} - {}",
        extract_error(stdout, combined)
    ))
}

/// Classifies a successful winget exit into a specific result string.
fn classify_success(app_id: &str, stdout: &str) -> String {
    if stdout.contains("Successfully installed") || stdout.contains("successfully") {
        format!("SUCCESS:{app_id} - updated successfully")
    } else if stdout.contains("No applicable update found")
        || stdout.contains("No newer package versions")
    {
        format!("[i] {app_id} - already up to date")
    } else if stdout.contains("No package found") {
        format!("FAILURE:{app_id} - package not found")
    } else {
        format!("SUCCESS:{app_id} - completed")
    }
}

/// Extracts a concise error message from winget output.
fn extract_error(stdout: &str, combined: &str) -> String {
    let stderr_part = combined
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("Unknown error")
        .trim();

    let msg = if !stderr_part.is_empty() && stderr_part != stdout.trim() {
        stderr_part
    } else {
        stdout
            .lines()
            .filter(|l| !l.trim().is_empty())
            .next_back()
            .unwrap_or("Update failed")
            .trim()
    };

    let msg = if msg.len() > 100 { &msg[..100] } else { msg };
    msg.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_output() {
        let result = parse_winget_output("");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_no_updates() {
        let output = "No applicable upgrade found.\n";
        let result = parse_winget_output(output);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_sample_output() {
        let output = "\
Name                           Id                          Version        Available      Source
-------------------------------------------------------------------------------------------------
Microsoft Visual Studio Code   Microsoft.VisualStudioCode  1.85.0         1.85.1         winget
Google Chrome                  Google.Chrome               120.0.6099.109 120.0.6099.130 winget
2 upgrades available.";

        let result = parse_winget_output(output);
        assert!(result.is_ok());
        let apps = result.unwrap();
        assert_eq!(apps.len(), 2);

        assert_eq!(apps[0].name, "Microsoft Visual Studio Code");
        assert_eq!(apps[0].id, "Microsoft.VisualStudioCode");
        assert_eq!(apps[0].version, "1.85.0");
        assert_eq!(apps[0].available, "1.85.1");
        assert_eq!(apps[0].source, "winget");

        assert_eq!(apps[1].name, "Google Chrome");
        assert_eq!(apps[1].id, "Google.Chrome");
        assert_eq!(apps[1].version, "120.0.6099.109");
        assert_eq!(apps[1].available, "120.0.6099.130");
        assert_eq!(apps[1].source, "winget");
    }

    #[test]
    fn test_parse_multi_word_name() {
        let output = "\
Name                            Id                         Version   Available   Source
--------------------------------------------------------------------------------------
Microsoft Windows Terminal      Microsoft.WindowsTerminal  1.18.0    1.19.0      winget
";
        let result = parse_winget_output(output);
        assert!(result.is_ok());
        let apps = result.unwrap();
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "Microsoft Windows Terminal");
        assert_eq!(apps[0].id, "Microsoft.WindowsTerminal");
    }

    #[test]
    fn test_parse_short_line_ignored() {
        let output = "\
Name   Id   Version   Available   Source
-----------------------------------------
short";
        let result = parse_winget_output(output);
        assert!(result.is_ok());
        // "short" is too short to reach the Id column
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_safe_slice_clamping() {
        let s = "hello";
        assert_eq!(safe_slice(s, 0, 100), "hello");
        assert_eq!(safe_slice(s, 3, 100), "lo");
        assert_eq!(safe_slice(s, 100, 200), "");
    }

    #[test]
    fn test_classify_success_updated() {
        let result = classify_success("Test.App", "Successfully installed");
        assert!(result.starts_with("SUCCESS:"));
        assert!(result.contains("updated successfully"));
    }

    #[test]
    fn test_classify_success_already_up_to_date() {
        let result = classify_success("Test.App", "No applicable update found");
        assert!(result.starts_with("[i]"));
        assert!(result.contains("already up to date"));
    }

    #[test]
    fn test_classify_success_no_package() {
        let result = classify_success("Test.App", "No package found");
        assert!(result.starts_with("FAILURE:"));
    }

    #[test]
    fn test_classify_success_generic() {
        let result = classify_success("Test.App", "some other output");
        assert!(result.starts_with("SUCCESS:"));
        assert!(result.contains("completed"));
    }

    #[test]
    fn test_classify_update_result_needs_close() {
        let result = classify_update_result(
            "Test.App",
            false,
            "",
            "application must be closed before updating",
        );
        assert!(result.is_ok());
        assert!(result.unwrap().contains("[!]"));
    }

    #[test]
    fn test_classify_update_result_failure() {
        let result = classify_update_result("Test.App", false, "Some error", "Some error\n");
        assert!(result.is_err());
        assert!(result.unwrap_err().starts_with("FAILURE:"));
    }

    #[test]
    fn test_extract_error_truncates() {
        let long_msg = "x".repeat(200);
        let result = extract_error("", &long_msg);
        assert!(result.len() <= 100);
    }

    #[test]
    fn test_parse_winget_output_with_header_variations() {
        // Some winget versions have slightly different spacing
        let output = "\
Name                  Id                 Version  Available  Source
------------------------------------------------------------------
Notepad++             Notepad++.Notepad++  8.5.0   8.6.0     winget
";
        let result = parse_winget_output(output);
        assert!(result.is_ok());
        let apps = result.unwrap();
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "Notepad++");
        assert_eq!(apps[0].id, "Notepad++.Notepad++");
    }

    #[test]
    fn test_sanitize_output_strips_cr_progress() {
        // Simulate winget's progress spinner: \r-delimited segments on one line.
        let raw = "\r   - \r   \\ \rName           Id             Version  Available  Source";
        let sanitized = sanitize_output(raw);
        assert_eq!(
            sanitized,
            "Name           Id             Version  Available  Source"
        );
    }

    #[test]
    fn test_sanitize_output_preserves_clean_lines() {
        let output = "Name  Id  Version\n------\nApp  A.A  1.0";
        let sanitized = sanitize_output(output);
        assert_eq!(sanitized, output);
    }

    #[test]
    fn test_parse_with_cr_prefixed_header() {
        // Real-world: header line has spinner prefix via \r.
        let header = "\r   - \r   \\ \rName                               Id                                     Version          Available        Source";
        let sep = "--------------------------------------------------------------------------------------------------------------";
        let data = "Google Chrome                      Google.Chrome                          120.0.6099.109   120.0.6099.130   winget";
        let footer = "1 upgrades available.";

        let output = format!("{header}\n{sep}\n{data}\n{footer}");
        let result = parse_winget_output(&output);
        assert!(result.is_ok(), "should parse output with \\r-prefixed header");
        let apps = result.unwrap();
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "Google Chrome");
        assert_eq!(apps[0].id, "Google.Chrome");
    }

    #[test]
    fn test_unwrap_output_no_wrapping() {
        let output = "\
Name       Id          Version   Available   Source
---------------------------------------------------
Notepad++  Notepad++.N 8.5.0     8.6.0       winget
1 upgrades available.";
        // No wrapping detected — should return as-is
        let unwrapped = unwrap_output(output);
        assert_eq!(unwrapped, output);
    }

    #[test]
    fn test_unwrap_output_with_wrapping() {
        // Simulate wrapping at width=50. Each wrapped line is exactly 50 chars;
        // continuation lines are shorter.
        // Header: "Name" at 0, "Id" at 20, "Version" at 35, "Available" at 46... (>50)
        // We'll build a 60-char header that wraps into 50+10.
        let header_full = format!("{:<20}{:<15}{:<11}{:<8}Source", "Name", "Id", "Version", "Available");
        // header_full is 60 chars. First 50 go on line 1, last 10 on line 2.
        let h1 = &header_full[..50];
        let h2 = &header_full[50..];
        let sep_full = "-".repeat(60);
        let s1 = &sep_full[..50];
        let s2 = &sep_full[50..];

        let output = format!("{h1}\n{h2}\n{s1}\n{s2}\ndata line");
        let unwrapped = unwrap_output(&output);
        assert!(
            unwrapped.contains("Version"),
            "Unwrapped should reconstruct 'Version' from wrapped header"
        );
        assert!(
            unwrapped.contains(&"-".repeat(60)),
            "Unwrapped separator should be the full 60-dash line"
        );
    }

    #[test]
    fn test_detect_wrap_width_consecutive_dashes() {
        let lines = vec!["some text", "----------------------------------------", "------", "data line"];
        assert_eq!(detect_wrap_width(&lines), Some(40));
    }

    #[test]
    fn test_detect_wrap_width_no_wrapping() {
        let lines = vec![
            "Name   Id   Version   Available   Source",
            "---------------------------------------------------",
            "data line",
        ];
        assert_eq!(detect_wrap_width(&lines), None);
    }

    #[test]
    fn test_parse_wrapped_output_end_to_end() {
        // Real-world scenario: winget output wrapped at 80 columns.
        // Full header is 114 chars → wraps into 80 + 34.
        // We must ensure proper column alignment so parsing works.
        //
        // Column positions: Name=0, Id=35, Version=74, Available=91, Source=108
        let header_full = format!(
            "{:<35}{:<39}{:<17}{:<17}{}",
            "Name", "Id", "Version", "Available", "Source"
        );
        // header_full = 114 chars (35+39+17+17+6)
        assert_eq!(header_full.len(), 114);

        let sep_full = "-".repeat(114);
        let data_full = format!(
            "{:<35}{:<39}{:<17}{:<17}{}",
            "Google Chrome",
            "Google.Chrome",
            "120.0.6099.109",
            "120.0.6099.130",
            "winget"
        );
        assert_eq!(data_full.len(), 114);

        let footer = "1 upgrades available.";

        // Wrap all lines at 80 columns.
        let wrap = |s: &str| -> String {
            if s.len() <= 80 {
                s.to_string()
            } else {
                format!("{}\n{}", &s[..80], &s[80..])
            }
        };

        let output = format!(
            "{}\n{}\n{}\n{footer}",
            wrap(&header_full),
            wrap(&sep_full),
            wrap(&data_full),
        );

        let result = parse_winget_output(&output);
        assert!(result.is_ok(), "parse should succeed on wrapped output");
        let apps = result.unwrap();
        assert_eq!(apps.len(), 1, "should find exactly 1 app");
        assert_eq!(apps[0].name, "Google Chrome");
        assert_eq!(apps[0].id, "Google.Chrome");
        assert_eq!(apps[0].version, "120.0.6099.109");
        assert_eq!(apps[0].available, "120.0.6099.130");
        assert_eq!(apps[0].source, "winget");
    }
}

