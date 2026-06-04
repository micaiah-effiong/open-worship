#!/bin/bash

cd "$(dirname "$0")"

BUNDLE="$(cd "$(dirname "$0")/.." && pwd)"
RESOURCES="$BUNDLE/Resources"

export GSETTINGS_SCHEMA_DIR="$RESOURCES/share/glib-2.0/schemas"
export GDK_PIXBUF_MODULE_FILE="$RESOURCES/lib/gdk-pixbuf-2.0/2.10.0/loaders.cache"
export RESOURCE_FILE="$RESOURCES/share/resources.gresource"

APP=./openworship

exec "$APP" "$@"
