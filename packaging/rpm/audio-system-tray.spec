Name:           audio-system-tray
Version:        0.1.0
Release:        1%{?dist}
Summary:        Native PipeWire/PulseAudio audio sink switcher system tray

License:        MIT
URL:            https://github.com/hakumai23/AudioSystemTray
# In a local development build, we assume the sources are copied to the build directory.
Source0:        audio-system-tray-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust

%description
Native Status Notifier Item (SNI) tray icon daemon that dynamically lists
and switches default audio output sinks. Works with Waybar, Hyprland, GNOME,
KDE, XFCE, and other desktop environments.

%prep
# No-op for local build if we prepare source archive, otherwise normal setup
%setup -q

%build
cargo build --release --locked

%install
rm -rf $RPM_BUILD_ROOT
install -d $RPM_BUILD_ROOT/%{_bindir}
install -d $RPM_BUILD_ROOT/%{_datadir}/applications

install -m 755 target/release/audio-system-tray $RPM_BUILD_ROOT/%{_bindir}/audio-system-tray
install -m 644 audio-system-tray.desktop $RPM_BUILD_ROOT/%{_datadir}/applications/audio-system-tray.desktop

%files
%{_bindir}/audio-system-tray
%{_datadir}/applications/audio-system-tray.desktop

%changelog
* Sat May 30 2026 hakumai23 <hakumai23@example.com> - 0.1.0-1
- Initial packaging release.
