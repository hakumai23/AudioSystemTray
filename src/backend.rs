use std::io;
use std::process::Command;

// ─── Data types ──────────────────────────────────────────────────────────────

/// A single audio output sink discovered from the backend status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioSink {
    /// Unique identifier for the sink (numeric ID for wpctl, symbolic name for pactl).
    pub id: String,
    /// Human-readable device name.
    pub name: String,
    /// `true` when this is the current default sink.
    pub is_default: bool,
}

/// Abstract backend interface to support multiple Linux audio servers.
pub trait AudioBackend: Send + Sync {
    /// Query all available audio sinks from the audio server.
    fn get_sinks(&self) -> io::Result<Vec<AudioSink>>;
    /// Set the default audio sink.
    fn set_default(&self, id: &str) -> io::Result<()>;
    /// Identify the backend name.
    fn name(&self) -> &'static str;
}

// ─── Wpctl Backend ───────────────────────────────────────────────────────────

pub struct WpctlBackend;

impl AudioBackend for WpctlBackend {
    fn name(&self) -> &'static str {
        "wpctl"
    }

    fn get_sinks(&self) -> io::Result<Vec<AudioSink>> {
        let output = Command::new("wpctl").arg("status").output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("wpctl status failed: {stderr}"),
            ));
        }
        let text = String::from_utf8_lossy(&output.stdout);
        Ok(parse_wpctl_sinks(&text))
    }

    fn set_default(&self, id: &str) -> io::Result<()> {
        let status = Command::new("wpctl")
            .args(["set-default", id])
            .status()?;
        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("wpctl set-default {id} failed with exit code: {status}"),
            ));
        }
        Ok(())
    }
}

/// Parse the text output of `wpctl status` and return every audio sink found.
pub fn parse_wpctl_sinks(output: &str) -> Vec<AudioSink> {
    let mut sinks = Vec::new();
    let mut in_sinks_section = false;

    for raw_line in output.lines() {
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

        let content = strip_tree_glyphs(raw_line);
        let content = content.trim();

        if content.is_empty() {
            continue;
        }

        let (is_default, rest) = if let Some(r) = content.strip_prefix('*') {
            (true, r.trim_start())
        } else {
            (false, content)
        };

        let Some((id_token, after_id)) = rest.split_once('.') else {
            continue;
        };

        let id = id_token.trim().to_string();
        if id.parse::<u32>().is_err() {
            continue;
        }

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

fn strip_tree_glyphs(line: &str) -> String {
    line.chars()
        .filter(|c| !matches!(*c, '│' | '├' | '─' | '└' | '┤'))
        .collect()
}

// ─── Pactl Backend ───────────────────────────────────────────────────────────

pub struct PactlBackend;

impl PactlBackend {
    /// Retrieve the default sink name using `pactl get-default-sink` or falling back to `pactl info`.
    fn get_default_sink_name(&self) -> Option<String> {
        // Try pactl get-default-sink (modern PulseAudio / pipewire-pulse)
        if let Ok(output) = Command::new("pactl").arg("get-default-sink").output() {
            if output.status.success() {
                let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }

        // Fallback to pactl info for older systems
        if let Ok(output) = Command::new("pactl").arg("info").output() {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                for line in text.lines() {
                    if let Some(rest) = line.strip_prefix("Default Sink:") {
                        let name = rest.trim().to_string();
                        if !name.is_empty() {
                            return Some(name);
                        }
                    }
                }
            }
        }

        None
    }
}

impl AudioBackend for PactlBackend {
    fn name(&self) -> &'static str {
        "pactl"
    }

    fn get_sinks(&self) -> io::Result<Vec<AudioSink>> {
        // Set locale to C to ensure predictable English output from pactl
        let output = Command::new("pactl")
            .env("LC_ALL", "C")
            .args(["list", "sinks"])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("pactl list sinks failed: {stderr}"),
            ));
        }

        let text = String::from_utf8_lossy(&output.stdout);
        let default_sink_name = self.get_default_sink_name();
        Ok(parse_pactl_sinks(&text, default_sink_name.as_deref()))
    }

    fn set_default(&self, id: &str) -> io::Result<()> {
        let status = Command::new("pactl")
            .args(["set-default-sink", id])
            .status()?;
        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("pactl set-default-sink {id} failed with exit code: {status}"),
            ));
        }
        Ok(())
    }
}

/// Parse the text output of `pactl list sinks`.
pub fn parse_pactl_sinks(output: &str, default_sink_name: Option<&str>) -> Vec<AudioSink> {
    let mut sinks = Vec::new();
    let mut current_id = None;
    let mut current_name = None;
    let mut current_description = None;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Sink #") {
            // Push previous sink if found
            if let (Some(id), Some(name)) = (current_id.take(), current_name.take()) {
                let is_default = default_sink_name.map_or(false, |d| d == id);
                sinks.push(AudioSink {
                    id,
                    name: current_description.unwrap_or(name),
                    is_default,
                });
                current_description = None;
            }
            // Parse new index
            if let Some(idx_str) = trimmed.strip_prefix("Sink #") {
                if idx_str.trim().parse::<u32>().is_ok() {
                    // We temporarily use this index as ID, but will prefer the symbolic "Name" if available.
                }
            }
        } else if let Some(rest) = trimmed.strip_prefix("Name:") {
            current_id = Some(rest.trim().to_string());
            current_name = Some(rest.trim().to_string());
        } else if let Some(rest) = trimmed.strip_prefix("Description:") {
            current_description = Some(rest.trim().to_string());
        }
    }

    // Push the last sink in the stream
    if let (Some(id), Some(name)) = (current_id, current_name) {
        let is_default = default_sink_name.map_or(false, |d| d == id);
        sinks.push(AudioSink {
            id,
            name: current_description.unwrap_or(name),
            is_default,
        });
    }

    sinks
}

// ─── Auto-Detection ──────────────────────────────────────────────────────────

/// Try to detect the available audio backend on this system.
pub fn detect_backend() -> Option<Box<dyn AudioBackend>> {
    // Try wpctl first (standard PipeWire/WirePlumber)
    let wpctl_backend = WpctlBackend;
    if wpctl_backend.get_sinks().is_ok() {
        return Some(Box::new(wpctl_backend));
    }

    // Fall back to pactl (PulseAudio or pipewire-pulse compatibility)
    let pactl_backend = PactlBackend;
    if pactl_backend.get_sinks().is_ok() {
        return Some(Box::new(pactl_backend));
    }

    None
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_WPCTL_OUTPUT: &str = r#"PipeWire 'pipewire-0' [1.6.5, user@host, cookie:12345]
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

    const SAMPLE_PACTL_OUTPUT: &str = r#"Sink #0
	State: SUSPENDED
	Name: alsa_output.pci-0000_00_1f.3.analog-stereo
	Description: Built-in Audio Analog Stereo
	Driver: module-alsa-card.c
	Sample Specification: s16le 2ch 44100Hz
	Channel Map: front-left,front-right
	Owner Module: 6
	Mute: no
	Volume: front-left: 65536 / 100% / 0.00 dB,   front-right: 65536 / 100% / 0.00 dB
	        balance 0.00
	Base Volume: 65536 / 100% / 0.00 dB
	Monitor Source: alsa_output.pci-0000_00_1f.3.analog-stereo.monitor
	Latency: 0 usec, configured 0 usec
	Flags: HARDWARE HW_MUTE_CTRL HW_VOLUME_CTRL DECIBEL_VOLUME LATENCY SET_FORMATS 
	Properties:
		alsa.resolution_bits = "16"
		device.api = "alsa"
	Active Port: analog-output-speaker
	Formats:
		pcm

Sink #1
	State: RUNNING
	Name: bluez_output.00_11_22_33_44_55.a2dp-sink
	Description: JBL Tour One M3
	Driver: module-bluez5-device.c
	Volume: front-left: 32768 /  50% / -18.00 dB,   front-right: 32768 /  50% / -18.00 dB
"#;

    #[test]
    fn parses_wpctl_sinks_correctly() {
        let sinks = parse_wpctl_sinks(SAMPLE_WPCTL_OUTPUT);
        assert_eq!(sinks.len(), 4);
        assert_eq!(sinks[2].id, "107");
        assert_eq!(sinks[2].name, "JBL Tour One M3");
        assert_eq!(sinks[2].is_default, true);
    }

    #[test]
    fn parses_pactl_sinks_correctly() {
        let sinks = parse_pactl_sinks(SAMPLE_PACTL_OUTPUT, Some("bluez_output.00_11_22_33_44_55.a2dp-sink"));
        assert_eq!(sinks.len(), 2);
        
        assert_eq!(sinks[0].id, "alsa_output.pci-0000_00_1f.3.analog-stereo");
        assert_eq!(sinks[0].name, "Built-in Audio Analog Stereo");
        assert_eq!(sinks[0].is_default, false);

        assert_eq!(sinks[1].id, "bluez_output.00_11_22_33_44_55.a2dp-sink");
        assert_eq!(sinks[1].name, "JBL Tour One M3");
        assert_eq!(sinks[1].is_default, true);
    }
}
