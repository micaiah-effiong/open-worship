#!/bin/bash

BUILD_DIR="build/macos"
CURRENT_DIR=$(pwd)
BUNDLE_VERSION=$(cargo pkgid | cut -d "#" -f2)
BUNDLE_BUILD=$(date +"%Y%m%d%H%M")

# build bundle
cargo bundle --release --format osx

# clean build dir
if [ -d "$BUILD_DIR" ]; then 
	echo "Cleaning build directory"
	rm -r $BUILD_DIR
fi

mkdir -p $BUILD_DIR
cp -r target/release/bundle/osx/Openworship.app $BUILD_DIR
cp res/macos/open-worship.sh $BUILD_DIR/Openworship.app/Contents/MacOS

cd $BUILD_DIR

# link dylib
dylibbundler -od -b -x Openworship.app/Contents/MacOS/open-worship -d Openworship.app/Contents/Resources/lib -p @executable_path/../Resources/lib


# fix info.plist
cp res/macos/Info.plist Openworship.app/Contents
sed -i '' "s/%BUNDLE_VERSION%/$BUNDLE_VERSION/g" Openworship.app/Contents/Info.plist
sed -i '' "s/%BUNDLE_BUILD%/$BUNDLE_BUILD/g" Openworship.app/Contents/Info.plist


cp -r /opt/homebrew/share/glib-2.0 Openworship.app/Contents/Resources/share
cp -r /opt/homebrew/share/icons Openworship.app/Contents/Resources/share
cp -r /opt/homebrew/share/locale Openworship.app/Contents/Resources/share

mkdir -p Openworship.app/Contents/Resources/lib/gdk-pixbuf-2.0/2.10.0/loaders
cp -r /opt/homebrew/lib/gdk-pixbuf-2.0/2.10.0/loaders/*.so Openworship.app/Contents/Resources/lib/gdk-pixbuf-2.0/2.10.0/loaders 
# fix loader libpixbufloader_svg.so id 
svg_lib="/opt/homebrew/opt/librsvg/lib/gdk-pixbuf-2.0/2.10.0/loaders/libpixbufloader_svg.dylib" 
if [ -f $svg_lib ]; then
	# copy  $file to lib 
	cp $svg_lib Openworship.app/Contents/Resources/lib

	# bundle fix 
	dylibbundler -b -of -s /opt/homebrew/Cellar/librsvg/2.60.0/lib -x $svg_lib -d Openworship.app/Contents/Resources/lib -p @executable_path/../Resources/lib
	
	# change id
	install_name_tool -id @executable_path/../Resources/lib/libpixbufloader_svg.dylib Openworship.app/Contents/Resources/lib/gdk-pixbuf-2.0/2.10.0/loaders/libpixbufloader_svg.so
fi

for loader in Openworship.app/Contents/Resources/lib/gdk-pixbuf-2.0/2.10.0/loaders/*.so; do
	dylibbundler -b -of -s /opt/homebrew/Cellar/librsvg/2.60.0/lib -x $loader -d Openworship.app/Contents/Resources/lib -p @executable_path/../Resources/lib
done

cp -r /opt/homebrew/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache Openworship.app/Contents/Resources/lib/gdk-pixbuf-2.0/2.10.0
sed -i '' "s|$(brew --prefix)|/Applications/Openworship.app/Contents/Resources|g" Openworship.app/Contents/Resources/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache

# reset dir
cd $CURRENT_DIR
