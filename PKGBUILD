# Maintainer: Joaquin (Pato) Decima <joaquin.decima@vasak.net.ar>

pkgname=vpanel
pkgver=0.1.0
pkgrel=1
pkgdesc="Vasak OS panel"
arch=('x86_64')
url="https://github.com/VasakOS/vpanel"
license=('GPL3')
depends=(
    'gtk3'
    'libx11'
    'libxcb'
    'xcb-util-wm'
    'xcb-util-image'
    'wayland'
    'wayland-protocols'
    'wlroots'
    'gdk-pixbuf2'
    'cairo'
    'pango'
    'hicolor-icon-theme'
    'adwaita-icon-theme'
    'libnotify'
    'dbus'
    'xdg-utils'
    'xdg-desktop-portal'
    'xdg-desktop-portal-gtk'
)
makedepends=(
    'rust'
    'cargo'
    'bun'
    'git'
)
source=("git+${url}.git")
sha256sums=('SKIP')

prepare() {
    cd "$srcdir/$pkgname"
    bun install
}

build() {
    cd "$srcdir/$pkgname"
    bun run build
}

package() {
    cd "$srcdir/$pkgname"
    
    # Crear directorios necesarios
    install -dm755 "$pkgdir/usr/bin"
    install -dm755 "$pkgdir/usr/share/applications"
    install -dm755 "$pkgdir/usr/share/$pkgname"
    
    # Instalar el binario
    install -Dm755 "src-tauri/target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
    
    # Instalar archivos de la aplicación
    cp -r dist/* "$pkgdir/usr/share/$pkgname/"
    
    # Instalar el archivo .desktop
    cat > "$pkgdir/usr/share/applications/$pkgname.desktop" << EOF
[Desktop Entry]
Name=Vasak Panel
Comment=Vasak OS Panel
Exec=$pkgname
Icon=menu-editor
Type=Application
Categories=System;
StartupNotify=true
X-GNOME-AutoRestart=true
X-GNOME-Autostart=true
EOF

    # Instalar el archivo de autostart
    install -Dm644 "$pkgdir/usr/share/applications/$pkgname.desktop" \
        "$pkgdir/etc/xdg/autostart/$pkgname.desktop"
}

# vim:set ts=4 sw=4 et: 