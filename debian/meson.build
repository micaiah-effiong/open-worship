if get_option('bundle') == 'deb'
  deb_conf = configuration_data()
  deb_conf.set('VERSION', meson.project_version())
  configure_file(
    input: 'control.in',
    output: 'control',
    configuration: deb_conf,
    install: true,
    install_dir: '..' / 'DEBIAN', # install in destdir not destdir/prefix
  )
  configure_file(
    input: 'postinst',
    output: 'postinst',
    copy: true,
    install: true,
    install_dir: '..' / 'DEBIAN', # install in destdir not destdir/prefix
  )

endif
