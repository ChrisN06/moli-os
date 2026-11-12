#!/bin/sh
# Build the Android TV client from the current dashboard. npm dependencies and Android tools must be installed.
set -eu
: "${JAVA_HOME:?Set JAVA_HOME to JDK 17 or later}"
: "${ANDROID_JAR:?Set ANDROID_JAR to an Android 35+ platform jar}"
: "${ANDROID_BUILD_TOOLS:?Set ANDROID_BUILD_TOOLS to the Android SDK build-tools directory}"
: "${MOLI_TV_URL:?Set MOLI_TV_URL to the Moli server origin}"
: "${MOLI_TV_KEYSTORE:?Set MOLI_TV_KEYSTORE to the private signing keystore path}"
: "${MOLI_TV_STOREPASS_FILE:?Set MOLI_TV_STOREPASS_FILE to the signing password file path}"
if [ "$#" -ne 1 ]; then
  echo "Usage: sh mobile/tv/build.sh /absolute/path/to/moli-tv.apk" >&2
  exit 2
fi

source_dir=$(cd "$(dirname "$0")" && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/classes" "$work/dex" "$work/assets/web"
cp -R "$source_dir/res" "$work/res"
mkdir "$work/res/xml"

python3 - "$work" <<'PY'
import os
import re
import sys
from pathlib import Path
from urllib.parse import urlsplit

origin = urlsplit(os.environ["MOLI_TV_URL"])
if (origin.scheme not in {"http", "https"} or not origin.hostname
    or not re.fullmatch(r"[a-z0-9.-]+", origin.hostname)
    or origin.netloc != origin.hostname + (f":{origin.port}" if origin.port else "")
    or origin.path not in {"", "/"} or origin.query or origin.fragment):
    sys.exit("MOLI_TV_URL must be an HTTP(S) origin with a hostname and optional port")
work = Path(sys.argv[1])
(work / "assets/server.txt").write_text(f"{origin.scheme}://{origin.netloc}\n")
cleartext = ""
if origin.scheme == "http":
    cleartext = f'<domain-config cleartextTrafficPermitted="true"><domain>{origin.hostname}</domain></domain-config>'
(work / "res/xml/network_security.xml").write_text(
    f'<network-security-config><base-config cleartextTrafficPermitted="false"/>{cleartext}</network-security-config>\n')
PY

(
  cd "$source_dir/../../web"
  npm run build
  cp -R dist/. "$work/assets/web/"
)

export JAVA_HOME
"$JAVA_HOME/bin/javac" --release 8 -cp "$ANDROID_JAR" -d "$work/classes" "$source_dir/src/org/molios/tv/TvActivity.java"
"$ANDROID_BUILD_TOOLS/d8" --lib "$ANDROID_JAR" --min-api 30 --output "$work/dex" "$work/classes/org/molios/tv/"*.class
"$ANDROID_BUILD_TOOLS/aapt2" compile --dir "$work/res" -o "$work/res.zip"
"$ANDROID_BUILD_TOOLS/aapt2" link -I "$ANDROID_JAR" --manifest "$source_dir/AndroidManifest.xml" \
  -A "$work/assets" -o "$work/unsigned.apk" "$work/res.zip"
(cd "$work/dex" && zip -q "$work/unsigned.apk" classes.dex)
"$ANDROID_BUILD_TOOLS/zipalign" -f 4 "$work/unsigned.apk" "$work/aligned.apk"
"$ANDROID_BUILD_TOOLS/apksigner" sign --ks "$MOLI_TV_KEYSTORE" --ks-pass "file:$MOLI_TV_STOREPASS_FILE" --out "$1" "$work/aligned.apk"
"$ANDROID_BUILD_TOOLS/apksigner" verify "$1"
