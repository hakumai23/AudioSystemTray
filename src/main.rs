//! audio-system-tray — Native PipeWire sink switcher system tray daemon
//!
//! Implements a Status Notifier Item (SNI) tray icon using `ksni` that lets
//! the user switch the default audio output sink from a context menu populated
//! dynamically from `wpctl status`.

mod wpctl;

use ksni::{
    menu::StandardItem,
    MenuItem, Tray, TrayMethods,
};
use std::sync::{Arc, Mutex};
use wpctl::AudioSink;

// ─── Tray state ───────────────────────────────────────────────────────────────

struct AudioTray {
    /// Most-recently-discovered sinks.
    sinks: Vec<AudioSink>,
    /// Thread-safe reference to the tray's own handle to trigger updates.
    handle: Arc<Mutex<Option<ksni::Handle<AudioTray>>>>,
}

impl AudioTray {
    fn new(sinks: Vec<AudioSink>, handle: Arc<Mutex<Option<ksni::Handle<AudioTray>>>>) -> Self {
        Self { sinks, handle }
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

        let mut menu_items = vec![header, separator_top];

        if self.sinks.is_empty() {
            menu_items.push(
                StandardItem {
                    label: "(no audio sinks found)".into(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        } else {
            // Use StandardItem with Unicode indicators instead of CheckmarkItem/RadioGroup.
            // This bypasses bugs in Waybar and other status bars where DBusMenu toggle properties
            // fail to render correctly after dynamic state changes.
            for sink in &self.sinks {
                let id = sink.id;
                
                let indicator = if sink.is_default { "● " } else { "  " };
                let label_text = format!("{}{}", indicator, sink.name);

                let item = StandardItem {
                    label: label_text,
                    activate: Box::new(move |tray: &mut AudioTray| {
                        if let Err(e) = wpctl::set_default(id) {
                            eprintln!("[audio-system-tray] set-default {id} failed: {e}");
                            return;
                        }

                        // Optimistically update our state to avoid race conditions
                        // where `wpctl status` might not reflect the change instantly.
                        for s in &mut tray.sinks {
                            s.is_default = s.id == id;
                        }

                        // Trigger an update asynchronously on the Tokio executor
                        // to notify the status bar that the layout/states changed.
                        let handle_opt = tray.handle.lock().unwrap().clone();
                        if let Some(h) = handle_opt {
                            tokio::spawn(async move {
                                let _ = h.update(|_| {}).await;
                            });
                        }
                    }),
                    ..Default::default()
                };
                menu_items.push(item.into());
            }
        }

        let separator_mid = MenuItem::Separator;
        menu_items.push(separator_mid);

        let refresh = StandardItem {
            label: "↺  Refresh".into(),
            activate: Box::new(move |tray: &mut AudioTray| {
                match wpctl::get_sinks() {
                    Ok(fresh_sinks) => {
                        tray.sinks = fresh_sinks;

                        // Trigger update notification
                        let handle_opt = tray.handle.lock().unwrap().clone();
                        if let Some(h) = handle_opt {
                            tokio::spawn(async move {
                                let _ = h.update(|_| {}).await;
                            });
                        }
                    }
                    Err(e) => eprintln!("[audio-system-tray] refresh failed: {e}"),
                }
            }),
            ..Default::default()
        }
        .into();
        menu_items.push(refresh);

        let quit = StandardItem {
            label: "Quit".into(),
            icon_name: "application-exit".into(),
            activate: Box::new(|_| std::process::exit(0)),
            ..Default::default()
        }
        .into();
        menu_items.push(quit);

        menu_items
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

    let handle_shared = Arc::new(Mutex::new(None));
    let tray = AudioTray::new(sinks, handle_shared.clone());

    // Spawn the SNI tray service.
    let handle = match tray.spawn().await {
        Ok(h) => h,
        Err(e) => {
            eprintln!(
                "[audio-system-tray] Fatal: could not register SNI: {e}\n\
                 Make sure a StatusNotifierWatcher is running (e.g. Waybar with a tray module)."
            );
            std::process::exit(1);
        }
    };

    // Store the handle in our shared structure so the menu callback can trigger updates.
    *handle_shared.lock().unwrap() = Some(handle);

    eprintln!("[audio-system-tray] Running. Use your status bar tray to switch audio output.");
    std::future::pending::<()>().await;
}
