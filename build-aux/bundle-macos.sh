#!/usr/bin/env bash

APP_EXECUTABLE=$1 
BUNDLE_PATH=$2
PROJECT_ROOT=$3
PLIST=$4
DMG_SPEC=$5

APP_NAME="$(basename $APP_EXECUTABLE)"
DMG_DEST="$(dirname $BUNDLE_PATH)"

echo "APP_EXECUTABLE = $APP_EXECUTABLE"
echo "BUNDLE_PATH = $BUNDLE_PATH"
echo "PROJECT_ROOT = $PROJECT_ROOT"
echo "PLIST = $PLIST"
echo "DMG_SPEC = $DMG_SPEC"
echo "APP_NAME = $APP_NAME"
echo "DMG_DEST = $DMG_DEST"

function process_dependencies()
{
  local destdir=$1
  local file=$2
  local rpath=$3

  echo "Processing $file"

  local inst_prefix="$(brew --prefix)/*"

  local DEPS=$(dyld_info -dependents $file | tail -n +4)
  local process_list=""
  for dep in $DEPS; do
    if [[ $dep == $inst_prefix ]]; then
      dep_file=$(basename $dep)
      new_dep_file=$destdir/$dep_file
      if [ ! -f $new_dep_file ]; then
        # Not exist, do copy
        echo "  Copying $dep"
        cp -n $dep $destdir
      fi

      # Fix the dependency
      echo "  Patching $dep"
      install_name_tool -change $dep $rpath/$dep_file $file

      # Collect list of dependencies
      process_list="$new_dep_file $process_list"
    fi
  done

  # Recursively process dependencies
  for dep in $process_list; do
    process_dependencies $target $destdir $dep $rpath
  done
}

# 1. Create the bundle
mkdir -p $BUNDLE_PATH/Contents/{MacOS,Resources}
mkdir -p $BUNDLE_PATH/Contents/Resources/{lib,share}
cp $APP_EXECUTABLE $BUNDLE_PATH/Contents/MacOS

cp $PROJECT_ROOT/res/macos/openworship.icns $BUNDLE_PATH/Contents/Resources
cp $PROJECT_ROOT/res/macos/openworship.sh $BUNDLE_PATH/Contents/MacOS

cp $PLIST $BUNDLE_PATH/Contents/

mkdir -p $BUNDLE_PATH/Contents/Resources/share/glib-2.0
cp -r /opt/homebrew/share/glib-2.0/schemas $BUNDLE_PATH/Contents/Resources/share/glib-2.0

# 2. Copy and fix dependencies
destDir=$BUNDLE_PATH/Contents/Resources/lib
process_dependencies $target $destDir $BUNDLE_PATH/Contents/MacOS/openworship "@executable_path/../Resources/lib"

# 3. Copy loaders
mkdir -p $BUNDLE_PATH/Contents/Resources/lib/gdk-pixbuf-2.0/2.10.0/loaders
cp -r /opt/homebrew/lib/gdk-pixbuf-2.0/2.10.0/loaders/*.so $BUNDLE_PATH/Contents/Resources/lib/gdk-pixbuf-2.0/2.10.0/loaders

# 4. Fix loaders
for loader in $BUNDLE_PATH/Contents/Resources/lib/gdk-pixbuf-2.0/2.10.0/loaders/*.so; do
  process_dependencies $target $destDir $loader "@executable_path/../Resources/lib"
done

cp -r /opt/homebrew/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache $BUNDLE_PATH/Contents/Resources/lib/gdk-pixbuf-2.0/2.10.0
sed -i '' "s|$(brew --prefix)|/Applications/Openworship.app/Contents/Resources|g" $BUNDLE_PATH/Contents/Resources/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache

#################
echo "Code signing application..."

# Sign all dylib and so files
find "$BUNDLE_PATH/Contents/Resources/lib" \( -name '*.dylib' -o -name '*.so' \) \
  -type f -exec codesign --force --deep \
  --preserve-metadata=entitlements,requirements,flags,runtime \
  --entitlements $PROJECT_ROOT/res/macos/Entitlements.plist -s - {} \;

# Sign the app
codesign --force --deep --preserve-metadata=entitlements,requirements,flags,runtime \
  --entitlements $PROJECT_ROOT/res/macos/Entitlements.plist -s - "$BUNDLE_PATH"

echo "Creating DMG..."
version=$(cargo pkgid | cut -d "#" -f2 | sed "s/@/-v/")
DMG_BUNDLE="$DMG_DEST/$version.dmg"

npx appdmg@0.6.6 "$DMG_SPEC" "$DMG_BUNDLE"

echo "Code signing DMG..."
codesign --force --deep --preserve-metadata=entitlements,requirements,flags,runtime \
  -s - "$DMG_BUNDLE"

echo "=== Build complete! ==="
echo "Output: $DMG_BUNDLE"
