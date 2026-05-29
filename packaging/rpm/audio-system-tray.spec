Name:           audio-system-tray
Version:        0.1.0
Release:        1%{?dist}
Summary:        Native PipeWire/PulseAudio audio sink switcher system tray

License:        MIT
URL:            https://github.com/hakumai23/AudioSystemTray
BuildArch:      x86_64

# Passed in by `make rpm` via --define "_binary <path>" and "--define "_desktop <path>"
%global binary  %{?_binary}%{!?_binary:/usr/local/bin/audio-system-tray}
%global desktop %{?_desktop}%{!?_desktop:audio-system-tray.desktop}

%description
Native Status Notifier Item (SNI) tray icon daemon that dynamically lists
and switches default audio output sinks. Works with Waybar, Hyprland, GNOME,
KDE, XFCE, and other desktop environments.

%prep
# Nothing to prepare — binary is pre-built

%build
# Nothing to build — binary is pre-built

%install
rm -rf $RPM_BUILD_ROOT
install -d $RPM_BUILD_ROOT/%{_bindir}
install -d $RPM_BUILD_ROOT/%{_datadir}/applications

install -m 755 %{binary}  $RPM_BUILD_ROOT/%{_bindir}/audio-system-tray
install -m 644 %{desktop} $RPM_BUILD_ROOT/%{_datadir}/applications/audio-system-tray.desktop

%files
%{_bindir}/audio-system-tray
%{_datadir}/applications/audio-system-tray.desktop

%changelog
* Sat May 30 2026 hakumai23 <hakumai23@example.com> - 0.1.0-1
- Initial packaging release.
