#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PROFILE="release"
BUNDLE_DIR="$ROOT/target/$PROFILE/bundle/macos"
APP="$BUNDLE_DIR/FlashBee.app"

cargo build --profile "$PROFILE" -p flashbee

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$ROOT/target/$PROFILE/flashbee" "$APP/Contents/MacOS/flashbee"
cp "$ROOT/crates/flashbee-app/assets/icon.icns" "$APP/Contents/Resources/icon.icns"

cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>FlashBee</string>
  <key>CFBundleDisplayName</key><string>FlashBee</string>
  <key>CFBundleIdentifier</key><string>com.flashbee.app</string>
  <key>CFBundleExecutable</key><string>flashbee</string>
  <key>CFBundleIconFile</key><string>icon.icns</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>0.4.0</string>
  <key>CFBundleVersion</key><string>0.4.0</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSSupportsAutomaticGraphicsSwitching</key><true/>
</dict>
</plist>
PLIST

# Ad-hoc signing keeps the app launchable locally without a developer certificate.
codesign --force --deep --sign - "$APP" >/dev/null 2>&1 || true
echo "Built $APP"

if [[ "${1:-}" == "--dmg" ]]; then
  hdiutil create -volname FlashBee -srcfolder "$APP" -ov -format UDZO \
    "$BUNDLE_DIR/FlashBee.dmg" >/dev/null
  echo "Built $BUNDLE_DIR/FlashBee.dmg"
fi
