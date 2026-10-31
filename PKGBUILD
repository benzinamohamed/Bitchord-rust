# Maintainer: BitChord Rust Team
pkgname=bitchord-bin
pkgver=0.1.0
pkgrel=1
pkgdesc="Aesthetic YouTube Music desktop client with time-synced lyrics for Arch Linux"
arch=('x86_64')
url="https://github.com/kushagrasinghx/BitChord"
license=('GPL3')
depends=('alsa-lib' 'libxkbcommon' 'wayland' 'libglvnd')
provides=('bitchord')
conflicts=('bitchord')

package() {
    install -Dm755 "${srcdir}/../target/release/bitchord" "${pkgdir}/usr/bin/bitchord"
    install -Dm644 "${srcdir}/../bitchord.desktop" "${pkgdir}/usr/share/applications/bitchord.desktop"
    install -Dm644 "${srcdir}/../assets/logo.png" "${pkgdir}/usr/share/icons/hicolor/256x256/apps/bitchord.png"
}
