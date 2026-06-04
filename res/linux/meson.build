subdir('icons')

desktop_conf = configuration_data()
desktop_conf.set('APP_ID', application_id)
configure_file(
  input: '@0@.desktop.in'.format(base_id),
  output: '@0@.desktop'.format(application_id),
  configuration: desktop_conf,
  install: true,
  install_dir: datadir / 'applications',
)

service_conf = configuration_data()
service_conf.set('APP_ID', application_id)
service_conf.set('BINDIR', bindir)
configure_file(
  input: '@0@.service.in'.format(base_id),
  output: '@0@.service'.format(application_id),
  configuration: service_conf,
  install: true,
  install_dir: datadir / 'dbus-1' / 'services',
)

metainfo_conf = configuration_data()
metainfo_conf.set('APP_ID', application_id)
configure_file(
  input: '@0@.metainfo.xml.in'.format(base_id),
  output: '@0@.metainfo.xml'.format(application_id),
  configuration: metainfo_conf,
  install: true,
  install_dir: datadir / 'metainfo',
)

configure_file(
  input: '@0@.xml'.format(base_id),
  output: '@0@.xml'.format(application_id),
  copy: true,
  install: true,
  install_dir: datadir / 'mime' / 'packages' / application_id,
)

configure_file(
  input: 'application-x-openworship.png',
  output: 'application-x-openworship.png',
  copy: true,
  install: true,
  install_dir: datadir / 'icons' / 'hicolor' / '48x48' / 'mimetypes',
)

configure_file(
  input: 'openworship.png',
  output: 'openworship.png',
  copy: true,
  install: true,
  install_dir: datadir / 'icons' / 'hicolor' / '256x256' / 'apps',
)

# appimage

# deb
