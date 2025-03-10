#!/bin/bash

target="apple-darwin"
BUNDLE_VERSION=$(cargo pkgid | cut -d "#" -f2)
BUNDLE_BUILD=$(date +"%Y%m%d%H%M")
APP_PREFIX=target/bundle
rm deps.txt

function process_dependencies() {
  local target=$1
  local destdir=$2
  local file=$3
  local rpath=$4

  echo "Processing $file"

  local inst_prefix="$(brew --prefix)/*"

  # local DEPS=$(dyld_info -dependents $file | tail -n +4 | awk '$1 !~ /System\//')
  local DEPS=$(dyld_info -dependents $file | tail -n +4 | awk '$1 !~ /^(\/System|\/usr)/')

  # local DEPS=$(dyld_info -dependents $file | tail -n +4)

  echo -ne "DEPS > $file \n$DEPS \n" >> deps.txt

  local process_list=""
  for dep in $DEPS; do
    # Handle Frameworks
    if [[ "$dep" == *.framework/* ]]; then
      # Copy the entire framework
      cp -r "$(dirname "$dep")" "$destdir"
      # ... (patch rpaths within the framework)
    elif [[ "$dep" != /* ]]; then # Relative path (likely in the same dir)
      dep_file=$(basename $dep)
      cp -n "$file_dir/$dep" "$destdir"
    else
      dep_file=$(basename $dep)
      cp -n "$dep" "$destdir"
    fi

    new_dep_file="$destdir/$dep_file"
    install_name_tool -change "$dep" "@executable_path/../Resources/lib/$dep_file" "$file"
  done


  # for dep in $DEPS; do
  #   if [[ $dep == $inst_prefix ]]; then
  #     dep_file=$(basename $dep)
  #     new_dep_file=$destdir/$dep_file
  #     if [ ! -f $new_dep_file ]; then
  #       # Not exist, do copy
  #       echo "  Copying $dep"
  #       cp -n $dep $destdir
  #     fi
  #
  #     # Fix the dependency
  #     echo "  Patching $dep"
  #     install_name_tool -change $dep $rpath/$dep_file $file
  #
  #     # Collect list of dependencies
  #     process_list="$new_dep_file $process_list"
  #   fi
  # done

  # Recursively process dependencies
  for dep in $process_list; do
    process_dependencies $target $destdir $dep $rpath
  done
}

# 1. Create the bundle
mkdir -p $APP_PREFIX/$target/OpenWorship.app/Contents/{MacOS,Resources}
mkdir -p $APP_PREFIX/$target/OpenWorship.app/Contents/Resources/{lib,share}
# cp target/$target/release/open-worship $APP_PREFIX/$target/OpenWorship.app/Contents/MacOS
# cp target/release/open-worship $APP_PREFIX/$target/OpenWorship.app/Contents/MacOS
cp -R target/debug/* $APP_PREFIX/$target/OpenWorship.app/Contents/MacOS

cp res/macos/open-worship.icns $APP_PREFIX/$target/OpenWorship.app/Contents/Resources
cp res/macos/open-worship.sh $APP_PREFIX/$target/OpenWorship.app/Contents/MacOS

cp res/macos/Info.plist $APP_PREFIX/$target/OpenWorship.app/Contents/
sed -i '' "s/%BUNDLE_VERSION%/$BUNDLE_VERSION/g" $APP_PREFIX/$target/OpenWorship.app/Contents/Info.plist
sed -i '' "s/%BUNDLE_BUILD%/$BUNDLE_BUILD/g" $APP_PREFIX/$target/OpenWorship.app/Contents/Info.plist

mkdir -p $APP_PREFIX/$target/OpenWorship.app/Contents/Resources/share/glib-2.0
cp -r /opt/homebrew/share/glib-2.0/schemas $APP_PREFIX/$target/OpenWorship.app/Contents/Resources/share/glib-2.0

# 2. Copy and fix dependencies
destDir=$APP_PREFIX/$target/OpenWorship.app/Contents/Resources/lib
process_dependencies $target $destDir $APP_PREFIX/$target/OpenWorship.app/Contents/MacOS/open-worship "@executable_path/../Resources/lib"

# 3. Copy loaders
mkdir -p $APP_PREFIX/$target/OpenWorship.app/Contents/Resources/lib/gdk-pixbuf-2.0/2.10.0/loaders
cp -r /opt/homebrew/lib/gdk-pixbuf-2.0/2.10.0/loaders/*.so $APP_PREFIX/$target/OpenWorship.app/Contents/Resources/lib/gdk-pixbuf-2.0/2.10.0/loaders

# 4. Fix loaders
for loader in $APP_PREFIX/$target/OpenWorship.app/Contents/Resources/lib/gdk-pixbuf-2.0/2.10.0/loaders/*.so; do
  process_dependencies $target $destDir $loader "@executable_path/../Resources/lib"
done

cp -r /opt/homebrew/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache $APP_PREFIX/$target/OpenWorship.app/Contents/Resources/lib/gdk-pixbuf-2.0/2.10.0
sed -i '' "s|$(brew --prefix)|/Applications/OpenWorship.app/Contents/Resources|g" $APP_PREFIX/$target/OpenWorship.app/Contents/Resources/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache

