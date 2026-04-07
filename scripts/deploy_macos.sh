#!/bin/bash

# depenency library:
# Make a .app file: https://gist.github.com/oubiwann/453744744da1141ccc542ff75b47e0cf
# Make a .dmg file: https://github.com/LinusU/node-appdmg
# Can't find library: https://www.jianshu.com/p/441a7553700f

PROJECTDIR="$( cd "$(dirname "$0")/../" ; pwd -P )"

# if it is a git dirctory, get git root path
if $(git rev-parse --is-inside-work-tree 2>/dev/null); then
    PROJECTDIR="$(git rev-parse --show-toplevel)"
fi

TARGETDIR="${PROJECTDIR}/build/openworship"
VERSION="$(cargo pkgid | cut -d "@" -f2)"
# export VERSION
echo "VERSION=$VERSION"
echo "PROJECTDIR=$PROJECTDIR"
echo "TARGETDIR=$TARGETDIR"

BUNDLE_VERSION=$(cargo pkgid | cut -d "@" -f2)
BUNDLE_BUILD=$(date +"%Y%m%d%H%M")

# rebuild app release version
rm -rf "${PROJECTDIR}/build"
# meson --prefix=$TARGETDIR --buildtype=release build
# ninja -C "${PROJECTDIR}/build" install

# copy app data files to target dir
echo -n "Copy app data files......"
mkdir -p "${TARGETDIR}/bin"
mkdir -p "${TARGETDIR}/etc"
mkdir -p "${TARGETDIR}/include"
mkdir -p "${TARGETDIR}/lib/plugin"
mkdir -p "${TARGETDIR}/share/doc"
mkdir -p "${TARGETDIR}/share/themes"
mkdir -p "${TARGETDIR}/share/glib-2.0/schemas"
mkdir -p "${TARGETDIR}/share/licenses/openworship"
mkdir -p "${TARGETDIR}/share/icons/hicolor/scalable/apps"
echo "[done]"

function lib_dependency_copy
{
  local target=$1
  local folder=$2

  lib_dir="$( cd "$( dirname "$1" )" >/dev/null 2>&1 && pwd )"
  libraries="$(otool -L $target | grep "/*.*dylib" -o | xargs)"
  for lib in $libraries; do
    if [[ '/usr/lib/' != ${lib:0:9} && '/System/Library/' != ${lib:0:16} ]]; then
      if [[ '@' == ${lib:0:1} ]]; then
        if [[ '@loader_path' == ${lib:0:12} ]]; then
          cp -nL "${lib/@loader_path/$lib_dir}" $folder
        else
          echo "Unsupport path: $lib"
        fi
      else
        cp -nL $lib $folder
      fi
    fi  
  done
}

function lib_dependency_analyze
{
  # This function use otool to analyze library dependency.
  # then copy the dependency libraries to destination path

  local library_dir=$1
  local targets_dir=$2

  libraries="$(find $library_dir -name \*.dylib -o -name \*.so -type f)"
  for lib in $libraries; do
      lib_dependency_copy $lib $targets_dir
      # otool -L $lib | grep "/usr/local/*.*dylib" -o | xargs -I{} cp -n "{}" "$targets_dir"
  done
}

# copy app dependency library to target dir
echo -n "Copy app dependency library......"

lib_dependency_copy /opt/homebrew/opt/glib/lib/libglib-2.0.0.dylib "${TARGETDIR}/bin"
lib_dependency_copy /opt/homebrew/opt/glib/lib/libgobject-2.0.0.dylib "${TARGETDIR}/bin"
lib_dependency_copy /opt/homebrew/opt/glib/lib/libgio-2.0.0.dylib "${TARGETDIR}/bin"
lib_dependency_copy /opt/homebrew/opt/gtk4/lib/libgtk-4.1.dylib "${TARGETDIR}/bin"
lib_dependency_copy /opt/homebrew/opt/cairo/lib/libcairo.2.dylib "${TARGETDIR}/bin"
lib_dependency_copy /opt/homebrew/opt/json-glib/lib/libjson-glib-1.0.0.dylib "${TARGETDIR}/bin"
lib_dependency_copy /opt/homebrew/opt/libunistring/lib/libunistring.5.dylib "${TARGETDIR}/bin"
lib_dependency_copy /opt/homebrew/opt/cairo/lib/libcairo-script-interpreter.2.dylib "${TARGETDIR}/bin"
lib_dependency_copy /opt/homebrew/opt/gettext/lib/libgettextsrc-1.0.dylib "${TARGETDIR}/bin"
lib_dependency_copy /opt/homebrew/opt/harfbuzz/lib/libharfbuzz-icu.0.dylib "${TARGETDIR}/bin"

# lib_dependency_copy ${TARGETDIR}/bin/libsoup-2.4.1.dylib "${TARGETDIR}/bin"
# lib_dependency_copy ${TARGETDIR}/bin/libgtksourceview-4.0.dylib "${TARGETDIR}/bin"
# lib_dependency_copy /usr/local/lib/libgnutls-dane.0.dylib "${TARGETDIR}/bin"

export PKG_CONFIG_PATH="$(brew --prefix icu4c)/lib/pkgconfig"
icu_version="$(pkg-config icu-io --modversion)"
icu_path="$(brew --prefix icu4c)"
lib_dependency_copy "$icu_path/lib/libicuio.${icu_version:0:2}.dylib" "${TARGETDIR}/bin"
lib_dependency_copy "$icu_path/lib/libicutu.${icu_version:0:2}.dylib" "${TARGETDIR}/bin"


cp -fL "${PROJECTDIR}/target/aarch64-apple-darwin/production/openworship" "${TARGETDIR}/bin/"
cp -fL "${PROJECTDIR}/res/macos/openworship.sh" "${TARGETDIR}/bin/launcher.sh"

cp -fL /opt/homebrew/opt/glib/lib/libgirepository-2.0.0.dylib "${TARGETDIR}/bin"
cp -fL /opt/homebrew/opt/librsvg/lib/librsvg-2.2.dylib "${TARGETDIR}/bin"
cp -fL /opt/homebrew/opt/glib/lib/libgthread-2.0.0.dylib "${TARGETDIR}/bin"
cp -fL /opt/homebrew/opt/gmp/lib/libgmpxx.4.dylib "${TARGETDIR}/bin"

# cp -f /usr/local/lib/libgtkmacintegration-gtk3.2.dylib "${TARGETDIR}/bin"
# cp -f /usr/local/lib/libcroco-0.6.3.dylib "${TARGETDIR}/bin"
# cp -f /usr/local/lib/p11-kit-proxy.dylib "${TARGETDIR}/bin"
echo "[done]"

# copy GDBus/Helper and dependencies files
echo -n "Copy GDBus/Helper and dependencies......"
cp -fL /opt/homebrew/Cellar/glib/2.88.0/bin/gdbus "${TARGETDIR}/bin"
cp -fL /opt/homebrew/opt/gdk-pixbuf/bin/gdk-pixbuf-query-loaders "${TARGETDIR}/bin"
lib_dependency_copy ${TARGETDIR}/bin/gdbus "${TARGETDIR}/bin"
lib_dependency_copy ${TARGETDIR}/bin/gdk-pixbuf-query-loaders "${TARGETDIR}/bin"
echo "[done]"

# copy GTK runtime dependencies resource
echo -n "Copy GTK runtime resource......"
cp -rfL /opt/homebrew/lib/gio "${TARGETDIR}/lib/"
cp -rfL /opt/homebrew/lib/gtk-4.0 "${TARGETDIR}/lib/"
cp -rfL /opt/homebrew/lib/gdk-pixbuf-2.0 "${TARGETDIR}/lib/"
cp -rfL /opt/homebrew/lib/girepository-1.0 "${TARGETDIR}/lib/"
cp -rfL /opt/homebrew/opt/gtk4 "${TARGETDIR}/etc/"

# cp -rf /usr/local/lib/libgda-5.0 "${TARGETDIR}/lib/"
# Avoid override the latest locale file
cp -rL /opt/homebrew/share/locale "${TARGETDIR}/share/"
cp -rL /opt/homebrew/share/icons "${TARGETDIR}/share/"
cp -rL /opt/homebrew/share/fontconfig "${TARGETDIR}/share/"
# cp -rfL /usr/local/share/themes/Mac "${TARGETDIR}/share/themes/"
# cp -rfL /usr/local/share/themes/Default "${TARGETDIR}/share/themes/"
# cp -rfL /usr/local/share/gtksourceview-4 "${TARGETDIR}/share/"
cp -rfL /opt/homebrew/share/glib-2.0/schemas "${TARGETDIR}/share/glib-2.0/schemas"
cp "${PROJECTDIR}/data/resources/com.openworship.app.gschema.xml" "${TARGETDIR}/share/glib-2.0/schemas"
glib-compile-schemas "${TARGETDIR}/share/glib-2.0/schemas"

# find "${TARGETDIR}/bin" -type f -path '*.dll.a' -exec rm '{}' \;
lib_dependency_analyze ${TARGETDIR}/lib ${TARGETDIR}/bin
lib_dependency_analyze ${TARGETDIR}/bin ${TARGETDIR}/bin
echo "[done]"

# copy app icons and license files to target dir
# echo -n "Copy app icon(svg) files......"
# cp -f "${PROJECTDIR}/data/assets/openworship.ico" "${TARGETDIR}/bin"
# cp -f "${PROJECTDIR}/data/assets/openworship.svg" "${TARGETDIR}/share/icons/hicolor/scalable/apps"
# echo "[done]"


# download license file: LGPL-3.0
echo -n "Downloading the remote license file......"
cp -f "${PROJECTDIR}/LICENSE" "${TARGETDIR}/share/licenses/openworship"
if [ ! -f "${TARGETDIR}/share/licenses/openworship/LICENSE" ]; then
  curl "https://raw.githubusercontent.com/micaiah-effiong/open-worship/refs/heads/main/LICENSE" -o "${TARGETDIR}/share/licenses/openworship/LICENSE"
  if [ $? -eq 0 ]; then
    echo "[done]"
  else
    echo "[failed]"
  fi
else
  echo "[done]"
fi

echo "make macos executable file(.app)......"
cd "${PROJECTDIR}/build"
cp "${PROJECTDIR}/res/macos/Info.plist" "${PROJECTDIR}/build"
cp "${PROJECTDIR}/res/macos/Entitlements.plist" "${PROJECTDIR}/build"
sed -i '' "s/%BUNDLE_VERSION%/$BUNDLE_VERSION/g" "${PROJECTDIR}/build/Info.plist"
sed -i '' "s/%BUNDLE_BUILD%/$BUNDLE_BUILD/g"  "${PROJECTDIR}/build/Info.plist"
cp "${PROJECTDIR}/res/macos/openworship.icns" "${PROJECTDIR}/build/openworship.icns"
$PROJECTDIR/scripts/mac_app_pack.sh --path "${TARGETDIR}" --name "openworship" --info "Info.plist" --icons "openworship.icns"
if [ $? -eq 0 ]; then
  echo "[done]"
  else
  echo "[failed]"
fi

# make installer package
echo "make macos installer(.dmg)......"
cp "${PROJECTDIR}/res/macos/spec-aarch64-apple-darwin.json" openworship_dmg.json
cp "${PROJECTDIR}/res/macos/background.png" "${PROJECTDIR}/build"

npx appdmg@0.6.6 openworship_dmg.json "openworship-${VERSION}.dmg"
if [ $? -eq 0 ]; then
  echo "[done]"
  else
  echo "[failed]"
fi

# # make portable package
# echo -n "make macos portable......"
# tar czf "${PROJECTDIR}/build/openworship-${VERSION}-macos.tar.gz" -C "${PROJECTDIR}/build/" openworship.app
# if [ $? -eq 0 ]; then
#   echo "[done]"
#   else
#   echo "[failed]"
# fi
#
