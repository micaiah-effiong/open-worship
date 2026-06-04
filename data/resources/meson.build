if host_machine.system() == 'darwin'
  resources = gnome.compile_resources(
    'resources',
    'resources.gresource.xml',
    gresource_bundle: true,
    source_dir: meson.current_source_dir(),
    install: true,
    install_dir: get_option('prefix') / app_name
    + '.app' / 'Contents' / 'Resources' / 'share',
  )

else
  resources = gnome.compile_resources(
    'resources',
    'resources.gresource.xml',
    gresource_bundle: true,
    source_dir: meson.current_source_dir(),
    install: true,
    install_dir: pkgdatadir,
  )
endif
