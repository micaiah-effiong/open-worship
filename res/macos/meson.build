app_dir = get_option('prefix') / app_name + '.app'
content_dir = app_dir / 'Contents'
macos_dir = content_dir / 'MacOS'
resources_dir = content_dir / 'Resources'
share_dir = resources_dir / 'share'
lib_dir = resources_dir / 'lib'

install_data(
  'openworship.sh',
  install_dir: macos_dir,
)

install_data(
  'openworship.icns',
  install_dir: resources_dir,
)

bundle_build = run_command('date', '+%Y%m%d%H%M', check: true).stdout().strip()
info_plist_conf = configuration_data()
info_plist_conf.set('BUNDLE_VERSION', meson.project_version())
info_plist_conf.set('BUNDLE_BUILD', bundle_build)
info_plist_file = configure_file(
  input: 'Info.plist', #.format(base_id),
  output: 'Info.plist',
  configuration: info_plist_conf,
  install: true,
  install_dir: content_dir,
)

dmg_file = 'dmg_spec.json'
dmgspec_conf = configuration_data()
dmgspec_conf.set(
  'BACKGROUND',
  meson.project_source_root() / 'res' / 'macos' / 'background.png',
)
dmgspec_conf.set('PROGRAM_PATH', get_option('prefix') / app_name + '.app')
dmgspec_file = configure_file(
  input: 'spec-apple-darwin.json.in',
  output: dmg_file,
  configuration: dmgspec_conf,
  install: true,
  install_dir: get_option('prefix'),
)

bin_path = bindir / 'openworship'

meson.add_install_script(
  meson.project_source_root() / 'build-aux' / 'bundle-macos.sh',
  bin_path,
  app_dir,
  meson.project_source_root(),
  info_plist_file.full_path(),
  dmgspec_file.full_path()
)

# # move app executable to macos folder
# meson.add_install_script(
#   'sh',
#   '-c', '''mkdir -p @1@ &&  cp "@0@" "@1@"'''.format(bin_path, macos_dir),
# )
#
# # copy glib schemas
# brew_prefix = run_command('brew', '--prefix', capture: true, check: true).stdout().strip()
# install_subdir(
#   brew_prefix / 'share' / 'glib-2.0' / 'schemas',
#   install_dir: share_dir / 'glib-2.0',
#   strip_directory: false,
#   follow_symlinks: true
# )
#
# # process deps
# rpath = '@executable_path/../Resources/lib'
# install_emptydir(lib_dir)
# meson.add_install_script(
#   meson.project_source_root() / 'build-aux' / 'process_mac_dependencies.sh',
#   lib_dir,
#   macos_dir / 'openworship',
#   rpath,
# )
#
# # copy loaders
# loaders_dir = brew_prefix / 'lib' / 'gdk-pixbuf-2.0' / '2.10.0' / 'loaders'
# so_files = run_command(
#   'sh',
#   '-c', 'find @0@ -maxdepth 1 -name "*.so"'.format(loaders_dir),
#   capture: true,
#   check: true,
# ).stdout().strip().split('\n')
# install_subdir(
#   loaders_dir,
#   install_dir: lib_dir / 'gdk-pixbuf-2.0' / '2.10.0',
#   strip_directory: false,
#   follow_symlinks: true
# )
#
# # process deps 2
# foreach so : so_files
#   meson.add_install_script(
#     meson.project_source_root() / 'build-aux' / 'process_mac_dependencies.sh',
#     lib_dir,
#     so,
#     rpath,
#   )
# endforeach
#
# # copy loaders.cache
# meson.add_install_script(
#   'cp',
#   '-c', brew_prefix / 'lib' / 'gdk-pixbuf-2.0' / '2.10.0' / 'loaders.cache',
#   lib_dir / 'gdk-pixbuf-2.0' / '2.10.0',
# )
#
# # update loaders.cache
# meson.add_install_script(
#   'sh',
#   '-c', '''
#     sed -i '' "s|@0@|/Applications/Openworship.app/Contents/Resources|g" @1@
#     '''.format(
#     brew_prefix,
#     lib_dir / 'gdk-pixbuf-2.0' / '2.10.0' / 'loaders.cache',
#   ),
# )
#
# # codesign
# message('Code signing application')
# entitlements_file = meson.project_source_root() / 'res' / 'macos' / 'Entitlements.plist'
# meson.add_install_script(
#   'sh',
#   '-c', '''
#   set -e
#   LIB_DIR="@0@"
#   ENTITLEMENTS="@1@"
#
#   find "$LIB_DIR" \( -name '*.dylib' -o -name '*.so' \) -type f | while read -r lib; do
#     echo "Codesigning: $lib"
#     codesign \
#       --force \
#       --deep \
#       --preserve-metadata=entitlements,requirements,flags,runtime \
#       --entitlements "$ENTITLEMENTS" \
#       -s - \
#       "$lib"
#   done
#   '''.format(lib_dir, entitlements_file),
# )
#
# meson.add_install_script(
#   'codesign',
#   '--force',
#   '--deep',
#   '--preserve-metadata=entitlements,requirements,flags,runtime',
#   '--entitlements', entitlements_file,
#   '-s',
#   '-', app_dir,
# )
#
# message('Creating DMG...')
#
# meson.add_install_script(
#   'npx',
#   'appdmg@0.6.6',
#   get_option('prefix') / dmg_file,
#   get_option('prefix') / 'openworship-@0@.dmg'.format(meson.project_version()),
# )
#
# message('Code signing DMG...')
# meson.add_install_script(
#   'sh',
#   '-c', 'codesign --force --deep --preserve-metadata=entitlements,requirements,flags,runtime -s - "@0@"'.format(
#     get_option('prefix') / 'openworship-@0@.dmg'.format(meson.project_version()),
#   ),
# )
