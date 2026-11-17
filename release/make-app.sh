#!/bin/bash
# Builds the release binary for Apple Silicon and Intel, joins the two into one
# universal executable, and wraps it in `target/release/Naran.app`, so it opens
# like any Mac app (no Terminal window, with its icon) on both kinds of Mac.
#
# Both Rust targets and `cargo-about` must be installed:
#   rustup target add aarch64-apple-darwin x86_64-apple-darwin
#   cargo install cargo-about --locked --features cli
# Set CARGO to pick a specific cargo (default: the one in PATH).
set -euo pipefail
cd "$(dirname "$0")/.."

CARGO=${CARGO:-cargo}
TARGETS=(aarch64-apple-darwin x86_64-apple-darwin)
APP=target/release/Naran.app

# The binary would carry the paths of this Mac (the user name, the folders of
# the sources and of the toolchain) in panic messages. They are replaced by
# neutral ones, and the symbols are stripped.
hide_local_paths() {
  RUSTFLAGS="--remap-path-prefix=$PWD=naran --remap-path-prefix=$HOME=~ -C strip=symbols"
  export RUSTFLAGS
}

build_binaries() {
  hide_local_paths
  for target in "${TARGETS[@]}"; do
    $CARGO build --release -p naran-app --target "$target"
  done
}

make_bundle_folders() {
  rm -rf "$APP"
  mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
}

join_binaries() {
  lipo -create -output "$APP/Contents/MacOS/naran-app" \
    target/aarch64-apple-darwin/release/naran-app \
    target/x86_64-apple-darwin/release/naran-app
}

copy_icon() {
  cp release/icon/Naran.icns "$APP/Contents/Resources/Naran.icns"
}

# Naran's own license, the Apache license of the GPUI parts, and the licenses
# of every dependency in the app, as their terms ask.
copy_licenses() {
  local resources="$APP/Contents/Resources"
  cp LICENSE "$resources/LICENSE"
  cp LICENSES/Apache-2.0.txt "$resources/LICENSE-Apache-2.0"
  $CARGO about generate --workspace -c release/licenses/about.toml \
    release/licenses/about.hbs -o "$resources/THIRD-PARTY-LICENSES.txt"
}

crate_version() {
  grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/'
}

write_info_plist() {
  local version
  version=$(crate_version)
  cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Naran</string>
  <key>CFBundleDisplayName</key><string>Naran</string>
  <key>CFBundleIdentifier</key><string>dev.naran.app</string>
  <key>CFBundleExecutable</key><string>naran-app</string>
  <key>CFBundleIconFile</key><string>Naran</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.developer-tools</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSPrincipalClass</key><string>NSApplication</string>
</dict>
</plist>
PLIST
}

# Apple Silicon refuses to run code that is not signed, and `lipo` drops the
# signature the linker added. An ad-hoc signature (no certificate) is enough
# to run on the Mac that built it; other Macs ask for approval on first open.
sign_bundle() {
  codesign --force --deep --sign - "$APP"
  codesign --verify --strict "$APP"
}

main() {
  build_binaries
  make_bundle_folders
  join_binaries
  copy_icon
  copy_licenses
  write_info_plist
  sign_bundle
  # Tell Finder the folder changed so the icon shows at once.
  touch "$APP"
  echo "$PWD/$APP"
}

main
