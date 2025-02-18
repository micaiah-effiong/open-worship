
$target="x86_64-pc-windows-msvc"

cargo build --release

Remove-Item target\wix\gtk4 -Recurse -Confirm:$false -ErrorAction SilentlyContinue

New-Item target\wix\gtk4\bin -ItemType Directory
New-Item target\wix\gtk4\lib\gdk-pixbuf-2.0\2.10.0\loaders -ItemType Directory
New-Item target\wix\gtk4\share\glib-2.0\schemas -ItemType Directory

Copy-Item -Path C:\gtk-build\gtk\x64\release\bin\*.dll -Destination target\wix\gtk4\bin
Copy-Item -Path C:\gtk-build\gtk\x64\release\bin\gdbus.exe -Destination target\wix\gtk4\bin
Copy-Item -Path C:\gtk-build\gtk\x64\release\bin\gspawn-win64-helper.exe -Destination target\wix\gtk4\bin
Copy-Item -Path C:\gtk-build\gtk\x64\release\bin\gspawn-win64-helper-console.exe -Destination target\wix\gtk4\bin

Copy-Item -Path C:\gtk-build\gtk\x64\release\lib\gdk-pixbuf-2.0\2.10.0\loaders\*.dll -Destination target\wix\gtk4\lib\gdk-pixbuf-2.0\2.10.0\loaders
Copy-Item -Path C:\gtk-build\gtk\x64\release\lib\gdk-pixbuf-2.0\2.10.0\loaders.cache -Destination target\wix\gtk4\lib\gdk-pixbuf-2.0\2.10.0

Copy-Item -Path C:\gtk-build\gtk\x64\release\share\glib-2.0\schemas\gschemas.compiled -Destination target\wix\gtk4\share\glib-2.0\schemas\gschemas.compiled

# TODO: Ideally something like this would have worked and we wouldn't need to hardcode stuff in `open-worship.wxs`: https://github.com/volks73/cargo-wix/issues/271
# & "C:\Program Files (x86)\WiX Toolset v3.11\bin\heat.exe" dir target\wix\gtk4 -gg -sfrag -template:fragment -out target\wix\gtk4.wxs -cg GTK -dr GTK

cargo wix --target $traget --no-build --nocapture

Remove-Item target\wix\gtk4 -Recurse -Confirm:$false -ErrorAction SilentlyContinue
