builddir := "builddir"
prefix := if os() == "linux" { "/usr" } else { `pwd` + "/" + builddir + "/bundle" }

setup profile="default" bundle="default":
  meson setup -Dprofile={{profile}} -Dbundle={{bundle}} {{builddir}} --prefix={{prefix}} 


setup-win profile="default" bundle="default":
  meson setup -Dprofile={{profile}} -Dbundle={{bundle}} {{builddir}} --prefix={{prefix}} --vsenv 

compile profile bundle: (setup profile bundle)
  meson compile -C {{builddir}} 

install-appimage: (setup "default" "deb")
  #!/usr/bin/env bash

  cd {{builddir}}

  LINUXDEPLOY_BINARY="linuxdeploy-$(uname -m).AppImage"
  if [ ! -f "./$LINUXDEPLOY_BINARY" ]; then
    echo "linuxdeploy not found!"
    wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/$LINUXDEPLOY_BINARY
    chmod +x ./$LINUXDEPLOY_BINARY
  fi

  if [ ! -f linuxdeploy-plugin-gtk.sh ]; then
    echo "plugin-gtk not found!"
    wget https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/3b67a1d1c1b0c8268f57f2bce40fe2d33d409cea/linuxdeploy-plugin-gtk.sh
    chmod +x ./linuxdeploy-plugin-gtk.sh
  fi

  cd -
  meson install -C {{builddir}} --destdir=AppDir

  cd {{builddir}}

  BUNDLE_DIR=$PWD/AppDir
  VERSION=$(cargo pkgid | cut -d "@" -f2)
  ./$LINUXDEPLOY_BINARY --appimage-extract
  DEPLOY_GTK_VERSION=4 NO_STRIP=1 ./squashfs-root/AppRun \
    --appdir AppDir \
    --plugin gtk \
    --output appimage
  mv Openworship-$(uname -m).AppImage openworship-$VERSION-$(uname -m).AppImage
  rm -r $BUNDLE_DIR

install-deb: (setup "default" "deb")
  #!/usr/bin/env bash

  BUNDLE_NAME="openworship_$(cargo pkgid | cut -d "@" -f2)"
  meson install -C {{builddir}} --destdir=$BUNDLE_NAME
  cd {{builddir}}
  BUNDLE_DIR="$PWD/$BUNDLE_NAME"

  echo "Setup control file Depends section"
  mkdir -p $BUNDLE_DIR/debian
  echo "Source: openworship" > "$BUNDLE_DIR/debian/control"
  cd $BUNDLE_DIR
  dpkg-shlibdeps -O "$BUNDLE_DIR/{{prefix}}/bin/openworship" 2>/dev/null | sed 's/shlibs:Depends=/Depends: /' >> "$BUNDLE_DIR/DEBIAN/control"
  cd -
  rm -r $BUNDLE_DIR/debian

  echo "Package debian app"
  fakeroot dpkg-deb --build $BUNDLE_DIR

  echo "Remove bundle dir"
  rm -r $BUNDLE_DIR
  echo "[Done]: Complete"

install-dmg: setup
  meson install -C builddir

install-exe: setup-win
  meson install -C builddir

debug:
    echo "PATH=$PATH"
    which link
    where.exe link

default: 
  just --list
