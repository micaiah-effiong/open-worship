# Build Debian package
# --profile production --no-build --no-strip --variant=modern
target=$(rustup target list | awk '/installed/ {print $1;}')
cargo deb --target $target --no-strip --profile release
# echo "[DEB]: $(ls target/$target)"
mv target/$target/debian/*.deb ./

# And build AppImage as well
if [ ! -f ./linuxdeploy-$(uname -m).AppImage ]; then
	echo "linuxdeploy not found!"
	wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-$(uname -m).AppImage
fi

if [ ! -f linuxdeploy-plugin-gtk.sh ]; then
	echo "plugin-gtk not found!"
	wget https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/3b67a1d1c1b0c8268f57f2bce40fe2d33d409cea/linuxdeploy-plugin-gtk.sh
fi

# chmod +x linuxdeploy*.AppImage linuxdeploy-plugin-gtk.sh
# NO_STRIP=1 ./linuxdeploy-$(uname -m).AppImage \
# 	--appdir AppDir \
# 	--plugin gtk \
# 	--executable target/$target/release/openworship \
# 	--desktop-file res/linux/com.openworship.app.desktop \
# 	--icon-file res/linux/openworship.png \
# 	--output appimage

chmod +x linuxdeploy*.AppImage linuxdeploy-plugin-gtk.sh
    
# Extract the AppImage
LINUXDEPLOY_BINARY="linuxdeploy-$(uname -m).AppImage"
./$LINUXDEPLOY_BINARY --appimage-extract

# Use the extracted binary
NO_STRIP=1 ./squashfs-root/AppRun \
	--appdir AppDir \
	--plugin gtk \
	--executable target/$target/release/openworship \
	--desktop-file res/linux/com.openworship.app.desktop \
	--icon-file res/linux/openworship.png \
	--output appimage

# Rename AppImage to be consistent with other files
version=$(grep -Po 'version = "\K.*?(?=")' -m 1 Cargo.toml)
mv Openworship-$(uname -m).AppImage openworship-$version-$(uname -m).AppImage
