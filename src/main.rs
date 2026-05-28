//! audio-system-tray — Native PipeWire sink switcher system tray daemon
//!
//! Implements a Status Notifier Item (SNI) tray icon using `ksni` that lets
//! the user switch the default audio output sink from a context menu populated
//! dynamically from `wpctl status`.

mod wpctl;

use ksni::{
    menu::{RadioGroup, RadioItem, StandardItem},
    MenuItem, Tray, TrayMethods,
};
use wpctl::AudioSink;

// ─── Tray state ───────────────────────────────────────────────────────────────

#[derive(Debug)]
struct AudioTray {
    /// Most-recently-discovered sinks.
    sinks: Vec<AudioSink>,
}

impl AudioTray {
    fn new(sinks: Vec<AudioSink>) -> Self {
        Self { sinks }
    }

    /// Index of the current default sink in `self.sinks`, or 0 if none found.
    fn default_index(&self) -> usize {
        self.sinks
            .iter()
            .position(|s| s.is_default)
            .unwrap_or(0)
    }
}

// ─── ksni::Tray implementation ────────────────────────────────────────────────

impl Tray for AudioTray {
    fn id(&self) -> String {
        "audio-system-tray".into()
    }

    fn title(&self) -> String {
        if let Some(sink) = self.sinks.iter().find(|s| s.is_default) {
            format!("🔊 {}", sink.name)
        } else {
            "🔊 Audio Output".into()
        }
    }

    fn icon_name(&self) -> String {
        "audio-volume-high".into()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let header = StandardItem {
            label: "Audio Output".into(),
            enabled: false,
            ..Default::default()
        }
        .into();

        let separator_top = MenuItem::Separator;

        // Build RadioItem list from discovered sinks.
        let radio_items: Vec<RadioItem> = self
            .sinks
            .iter()
            .map(|s| RadioItem {
                label: s.name.clone(),
                ..Default::default()
            })
            .collect();

        // Clone sink IDs so the callback closure can own them.
        let sink_ids: Vec<u32> = self.sinks.iter().map(|s| s.id).collect();

        let radio_group = if radio_items.is_empty() {
            StandardItem {
                label: "(no audio sinks found)".into(),
                enabled: false,
                ..Default::default()
            }
            .into()
        } else {
            RadioGroup {
                selected: self.default_index(),
                select: Box::new(move |tray: &mut AudioTray, idx: usize| {
                    let Some(&id) = sink_ids.get(idx) else { return };

                    // Execute synchronously to avoid nested async runtime panics.
                    if let Err(e) = wpctl::set_default(id) {
                        eprintln!("[audio-system-tray] set-default {id} failed: {e}");
                        return;
                    }

                    // Re-fetch sinks immediately.
                    match wpctl::get_sinks() {
                        Ok(fresh_sinks) => tray.sinks = fresh_sinks,
                        Err(e) => eprintln!("[audio-system-tray] refresh failed: {e}"),
                    }
                }),
                options: radio_items,
                ..Default::default()
            }
            .into()
        };

        let separator_mid = MenuItem::Separator;

        let refresh = StandardItem {
            label: "↺  Refresh".into(),
            activate: Box::new(move |tray: &mut AudioTray| {
                match wpctl::get_sinks() {
                    Ok(fresh_sinks) => tray.sinks = fresh_sinks,
                    Err(e) => eprintln!("[audio-system-tray] refresh failed: {e}"),
                }
            }),
            ..Default::default()
        }
        .into();

        let quit = StandardItem {
            label: "Quit".into(),
            icon_name: "application-exit".into(),
            activate: Box::new(|_| std::process::exit(0)),
            ..Default::default()
        }
        .into();

        vec![header, separator_top, radio_group, separator_mid, refresh, quit]
    }
}

// ─── Entry point ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    // Initial sink discovery.
    let sinks = match wpctl::get_sinks() {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "[audio-system-tray] Fatal: could not query wpctl: {e}\n\
                 Make sure WirePlumber is running."
            );
            std::process::exit(1);
        }
    };

    eprintln!(
        "[audio-system-tray] Started — {} sink(s) discovered.",
        sinks.len()
    );

    let tray = AudioTray::new(sinks);

    // Spawn the SNI tray service.
    let _handle = match tray.spawn().await {
        Ok(h) => h,
        Err(e) => {
            eprintln!(
                "[audio-system-tray] Fatal: could not register SNI: {e}\n\
                 Make sure a StatusNotifierWatcher is running (e.g. Waybar with a tray module)."
            );
            std::process::exit(1);
        }
    };

    eprintln!("[audio-system-tray] Running. Use your status bar tray to switch audio output.");
    std::future::pending::<()>().await;
}
