all:
	cargo build --release

install:
	install -Dm 755 target/release/vanilla_text /usr/bin/vanilla_text
	mkdir -p /usr/share/vanilla_text/ui
	install -Dm 755 ui/* /usr/share/vanilla_text/ui/
	install -Dm 644 assets/vanilla_text.desktop /usr/share/applications/vanilla_text.desktop
	install -Dm 644 assets/icon_48x48.png /usr/share/icons/hicolor/48x48/apps/vanilla_text.png
	install -Dm 644 assets/icon_64x64.png /usr/share/icons/hicolor/64x64/apps/vanilla_text.png
	install -Dm 644 assets/icon_128x128.png /usr/share/icons/hicolor/128x128/apps/vanilla_text.png
	gtk-update-icon-cache /usr/share/icons/hicolor

uninstall:
	rm /usr/bin/vanilla_text
	rm -r /usr/share/vanilla_text
	rm /usr/share/applications/vanilla_text.desktop
	rm /usr/share/icons/hicolor/48x48/apps/vanilla_text.png
	rm /usr/share/icons/hicolor/64x64/apps/vanilla_text.png
	rm /usr/share/icons/hicolor/128x128/apps/vanilla_text.png
	gtk-update-icon-cache /usr/share/icons/hicolor


