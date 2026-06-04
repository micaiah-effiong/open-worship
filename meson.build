project('openworship', 'rust', version: '0.0.1', meson_version: '>= 1.7')

gnome = import('gnome')
base_id = 'com.openworship.app'
app_name = 'Openworship'

dependency('gtk4', version: '>=4.14')
dependency('glib-2.0', version: '>=2.80.0')
dependency('gio-2.0', version: '>=2.80.0')
dependency('libadwaita-1', version: '>=1.6')
dependency('cairo', version: '>=1.2.4')
dependency('gdk-pixbuf-2.0', version: '>=2.22.0')
dependency('graphene-1.0', version: '>=1.5.4')
dependency('pango', version: '>=1.14.0')
dependency('openssl', version: '>=3.0.0')
dependency('zlib', version: '>=1.2.3')

is_dev = get_option('profile') == 'development'
if is_dev
  profile = 'Devel'
  application_id = '@0@.@1@'.format(base_id, profile)
  rust_target = 'debug'
else
  profile = ''
  application_id = base_id
  rust_target = 'production'
endif

bindir = get_option('prefix') / get_option('bindir')
localedir = get_option('prefix') / get_option('localedir')
datadir = get_option('prefix') / get_option('datadir')
pkgdatadir = datadir / meson.project_name()
libdir = get_option('prefix') / get_option('libdir')

subdir('data')
if host_machine.system() == 'linux'
  subdir('debian')
endif
subdir('src')
subdir('res')

if host_machine.system() == 'linux'
  gnome.post_install(
    gtk_update_icon_cache: true,
    glib_compile_schemas: true,
    update_desktop_database: true,
    update_mime_database: true,
  )
endif

# setup .cargo/config
cargo_conf = configuration_data()
cargo_conf.set('APP_ID', application_id)
cargo_conf.set('PROFILE', profile)
cargo_conf.set(
  'RESOURCE_FILE',
  meson.project_build_root() / 'data' / 'resources' / 'resources.gresource',
)
cargo_conf.set(
  'RESOURCE_SCHEMA',
  meson.project_build_root() / 'data' / '@0@.gschema.xml'.format(application_id),
)
cargo_conf.set(
  'GSETTINGS_SCHEMA_DIR',
  meson.project_build_root() / 'data',
)

configure_file(
  input: '.cargo/config.toml.in',
  output: 'config.toml',
  configuration: cargo_conf,
  install_dir: '.cargo',
)
