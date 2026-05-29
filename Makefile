PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
APPDIR ?= $(PREFIX)/share/applications
AUTOSTARTDIR ?= $(HOME)/.config/autostart

TARGET = x86_64-unknown-linux-musl
BINARY_STATIC = target/$(TARGET)/release/audio-system-tray
BINARY_NATIVE = target/release/audio-system-tray

.PHONY: all setup build build-native install clean test deb rpm arch packages

all: build

setup:
	rustup target add $(TARGET)

build:
	cargo build --release --target $(TARGET)

build-native:
	cargo build --release

test:
	cargo test

install:
	@if [ -f "$(BINARY_STATIC)" ]; then \
		echo "Installing static binary $(BINARY_STATIC)..."; \
		install -Dm755 "$(BINARY_STATIC)" "$(DESTDIR)$(BINDIR)/audio-system-tray"; \
	elif [ -f "$(BINARY_NATIVE)" ]; then \
		echo "Installing native binary $(BINARY_NATIVE)..."; \
		install -Dm755 "$(BINARY_NATIVE)" "$(DESTDIR)$(BINDIR)/audio-system-tray"; \
	else \
		echo "Error: Binary not found. Run 'make build' or 'make build-native' first."; \
		exit 1; \
	fi
	install -Dm644 audio-system-tray.desktop "$(DESTDIR)$(APPDIR)/audio-system-tray.desktop"
	@mkdir -p "$(AUTOSTARTDIR)"
	cp audio-system-tray.desktop "$(AUTOSTARTDIR)/audio-system-tray.desktop"
	@echo "Installation successful!"

deb:
	@bash packaging/debian/build-deb.sh

rpm:
	@echo "Building RPM package..."
	@mkdir -p target/rpm/SOURCES target/rpm/SPECS target/rpm/BUILD target/rpm/RPMS target/rpm/SRPMS
	@tar --transform 's,^,audio-system-tray-0.1.0/,' -czf target/rpm/SOURCES/audio-system-tray-0.1.0.tar.gz Cargo.toml Cargo.lock src/ Makefile audio-system-tray.desktop
	@cp packaging/rpm/audio-system-tray.spec target/rpm/SPECS/
	@rpmbuild -bb target/rpm/SPECS/audio-system-tray.spec --define "_topdir $(PWD)/target/rpm" || echo "Note: rpmbuild failed. Make sure rpm-build (rpmbuild) is installed."

arch:
	@echo "Building Arch Linux package..."
	@mkdir -p target/arch
	@if [ -f "$(BINARY_STATIC)" ]; then \
		cp "$(BINARY_STATIC)" target/arch/audio-system-tray; \
	elif [ -f "$(BINARY_NATIVE)" ]; then \
		cp "$(BINARY_NATIVE)" target/arch/audio-system-tray; \
	else \
		echo "Error: Binary not found. Building native release binary first..."; \
		cargo build --release; \
		cp "$(BINARY_NATIVE)" target/arch/audio-system-tray; \
	fi
	@cp audio-system-tray.desktop target/arch/
	@cp packaging/arch/PKGBUILD target/arch/
	@cd target/arch && makepkg -f -d || echo "Note: makepkg failed."

packages: deb rpm arch

clean:
	cargo clean
	rm -rf target/debian-pkg target/rpm target/arch
