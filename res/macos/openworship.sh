#!/bin/bash

cd "$(dirname "$0")"

export GSETTINGS_SCHEMA_DIR="/Applications/Openworship.app/Contents/Resources/share/glib-2.0/schemas"
export GDK_PIXBUF_MODULE_FILE="/Applications/Openworship.app/Contents/Resources/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache"

exec ./openworship
