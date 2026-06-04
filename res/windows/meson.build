configure_file(
  input: 'openworship.ico',
  output: 'openworship.ico',
  copy: true,
  install: true,
  install_dir: bindir
)

inno_conf = configuration_data() 
inno_conf.set('LICENSE', meson.project_source_root() / 'LICENSE')
inno_script = configure_file(
  input: 'inno.iss.in',
  output: 'inno.iss',
  configuration: inno_conf,
)
custom_target(
  'copy-inno-script',
  input: inno_script,
  output: 'inno-copy.stamp',
  command: [
    'bash', '-c',
    'cp "@INPUT@" "' + meson.project_build_root() + '/inno.iss" && touch @OUTPUT@',
  ],
  build_by_default: true,
)

windir =  'C:/gtk-build/gtk/x64/release/'

install_subdir(
  windir / 'share'/'icons',
  install_dir: datadir,
  strip_directory: false,
  follow_symlinks: true
)

install_subdir(
  windir / 'lib'/'gdk-pixbuf-2.0',
  install_dir: libdir,
  strip_directory: false,
  follow_symlinks: true
)

# meson.add_install_script(
#   'bash',
#   'cp',
#   windir / 'share'/'icons',
#   datadir
# )
#
# meson.add_install_script(
#   'bash',
#   windir / 'lib'/'gdk-pixbuf-2.0',
#   libdir
# )
