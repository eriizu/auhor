pkgname=author
pkgver=0.3.0
pkgrel=1
pkgdesc="add, remove, list project's author.txt"
arch=("x86_64")
url="https://github.com/eriizu/author"
license=("MIT")
makedepends=("rust" "cargo" "git")
source=("git+https://github.com/eriizu/author.git")
sha256sums=("SKIP")

build() {
  cd "$srcdir/$pkgname"
  cargo build --release --locked
}

package() {
  cd "$srcdir/$pkgname"
  install -Dm755 "target/release/author" "$pkgdir/usr/bin/author"
}
