project('openworship', 'rust', version: '0.0.1', meson_version: '>= 1.7')

gnome = import('gnome')
base_id = 'com.openworship.app'

dependency('gtk4', version: '>=4.18')
dependency('glib-2.0', version: '>=2.84.4')
dependency('gio-2.0', version: '>=2.84.4')

is_dev = get_option('profile') == 'development'
if is_dev
  profile = 'Devel'
  application_id = '@0@.@1@'.format(base_id, profile)
else
  profile = ''
  application_id = base_id
endif

bindir = get_option('prefix') / get_option('bindir')
localedir = get_option('prefix') / get_option('localedir')
datadir = get_option('prefix') / get_option('datadir')
pkgdatadir = datadir / meson.project_name()
libdir = get_option('prefix') / get_option('libdir')

subdir('data')
subdir('src')

gnome.post_install(
  gtk_update_icon_cache: true,
  glib_compile_schemas: true,
  update_desktop_database: true,
)
