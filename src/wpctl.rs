//! wpctl.rs — WirePlumber control interface
//!
//! Handles all interactions with `wpctl`: parsing `wpctl status` output into
//! structured [`AudioSink`] data and issuing sink-switch commands via
//! `wpctl set-default`.

use std::io;
use std::process::Command;

// ─── Data types ──────────────────────────────────────────────────────────────

/// A single PipeWire audio output sink discovered from `wpctl status`.
#[derive(Debug, Clone)]
pub struct AudioSink {
    /// Numeric PipeWire object ID used by `wpctl`.
    pub id: u32,
    /// Human-readable device name (trimmed, `[vol: …]` stripped).
    pub name: String,
    /// `true` when this is the current default sink (line starts with `*`).
    pub is_default: bool,
}

// ─── Parsing ─────────────────────────────────────────────────────────────────

/// Parse the text output of `wpctl status` and return every audio sink found
/// in the `Sinks:` section.
pub fn parse_sinks(output: &str) -> Vec<AudioSink> {
    let mut sinks = Vec::new();
    let mut in_sinks_section = false;

    for raw_line in output.lines() {
        // ── Section detection ──────────────────────────────────────────────
        let stripped_header = strip_tree_glyphs(raw_line);

        if stripped_header.ends_with("Sinks:") {
            in_sinks_section = true;
            continue;
        }

        if in_sinks_section
            && raw_line.contains("─ ")
            && stripped_header.ends_with(':')
        {
            break;
        }

        if !in_sinks_section {
            continue;
        }

        // ── Sink line parsing ──────────────────────────────────────────────
        let content = strip_tree_glyphs(raw_line);
        let content = content.trim();

        if content.is_empty() {
            continue;
        }

        // Detect and consume the default marker.
        let (is_default, rest) = if let Some(r) = content.strip_prefix('*') {
            (true, r.trim_start())
        } else {
            (false, content)
        };

        // The first token should be "<id>." followed by the name.
        let Some((id_token, after_id)) = rest.split_once('.') else {
            continue;
        };

        let Ok(id) = id_token.trim().parse::<u32>() else {
            continue;
        };

        // Strip trailing "[vol: …]" annotation and clean up whitespace.
        let name_raw = after_id.trim();
        let name = if let Some(vol_pos) = name_raw.rfind("[vol:") {
            name_raw[..vol_pos].trim().to_string()
        } else {
            name_raw.to_string()
        };

        if name.is_empty() {
            continue;
        }

        sinks.push(AudioSink { id, name, is_default });
    }

    sinks
}

/// Strip PipeWire tree-drawing Unicode characters from a line so we are left
/// with plain ASCII content.
fn strip_tree_glyphs(line: &str) -> String {
    line.chars()
        .filter(|c| !matches!(*c, '│' | '├' | '─' | '└' | '┤'))
        .collect()
}

// ─── Synchronous wpctl commands ──────────────────────────────────────────────

/// Spawn `wpctl status` synchronously, capture its output, and return the parsed sink list.
pub fn get_sinks() -> io::Result<Vec<AudioSink>> {
    let output = Command::new("wpctl")
        .arg("status")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("wpctl status failed: {stderr}"),
        ));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    Ok(parse_sinks(&text))
}

/// Execute `wpctl set-default <id>` synchronously to change the system default audio sink.
pub fn set_default(id: u32) -> io::Result<()> {
    let status = Command::new("wpctl")
        .args(["set-default", &id.to_string()])
        .status()?;

    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("wpctl set-default {id} failed with exit code: {status}"),
        ));
    }

    Ok(())
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_OUTPUT: &str = r#"PipeWire 'pipewire-0' [1.6.5, user@host, cookie:12345]
 └─ Clients:
        32. WirePlumber

Audio
 ├─ Devices:
 │      50. GP104 High Definition Audio Controller [alsa]
 │
 ├─ Sinks:
 │      56. Ryzen HD Audio Controller Analog Stereo [vol: 0.39]
 │      65. Easy Effects Sink                   [vol: 1.00]
 │  *  107. JBL Tour One M3                     [vol: 0.29]
 │     228. GP104 High Definition Audio Controller Digital Stereo (HDMI) [vol: 0.30]
 │
 ├─ Sources:
 │      57. Ryzen HD Audio Controller Analog Stereo [vol: 1.00]
 │      66. Easy Effects Source                 [vol: 1.00]
 │
"#;

    #[test]
    fn parses_four_sinks() {
        let sinks = parse_sinks(SAMPLE_OUTPUT);
        assert_eq!(sinks.len(), 4);
    }

    #[test]
    fn detects_default_sink() {
        let sinks = parse_sinks(SAMPLE_OUTPUT);
        let default_sinks: Vec<_> = sinks.iter().filter(|s| s.is_default).collect();
        assert_eq!(default_sinks.len(), 1);
        assert_eq!(default_sinks[0].id, 107);
        assert_eq!(default_sinks[0].name, "JBL Tour One M3");
    }
}
