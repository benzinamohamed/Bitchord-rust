# Maintainer: BitChord Rust Team
pkgname=bitchord-rust-bin
pkgver=0.1.0
pkgrel=1
pkgdesc="Aesthetic YouTube Music desktop client with time-synced lyrics for Arch Linux (Rust)"
arch=('x86_64')
url="https://github.com/kushagrasinghx/BitChord"
license=('GPL3')
depends=('webkit2gtk-4.1' 'alsa-lib' 'libxkbcommon' 'wayland' 'ffmpeg')
provides=('bitchord-rust')
conflicts=('bitchord-rust')

package() {
    install -Dm755 "${srcdir}/../target/release/bitchord" "${pkgdir}/usr/bin/bitchord-rust"
    install -Dm644 "${srcdir}/../bitchord-rust.desktop" "${pkgdir}/usr/share/applications/bitchord-rust.desktop"
    install -Dm644 "${srcdir}/../assets/logo.png" "${pkgdir}/usr/share/icons/hicolor/256x256/apps/bitchord-rust.png"
}
