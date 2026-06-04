subdir('resources')

if host_machine.system() == 'darwin'
  schema_dir = get_option('prefix') / app_name + '.app' / 'Contents' / 'Resources' / 'share' / 'glib-2.0' / 'schemas'

  gschema_conf = configuration_data()
  gschema_conf.set('APP_ID', application_id)
  configure_file(
    input: '@0@.gschema.xml.in'.format(base_id),
    output: '@0@.gschema.xml'.format(application_id),
    configuration: gschema_conf,
    install: true,
    install_dir: schema_dir,
  )

  meson.add_install_script(
    'glib-compile-schemas',
    schema_dir,
  )
else
  schema_dir = datadir / 'glib-2.0' / 'schemas'
  gschema_conf = configuration_data()
  gschema_conf.set('APP_ID', application_id)
  configure_file(
    input: '@0@.gschema.xml.in'.format(base_id),
    output: '@0@.gschema.xml'.format(application_id),
    configuration: gschema_conf,
    install: true,
    install_dir: schema_dir
  )

  if host_machine.system() == 'windows' 
    meson.add_install_script(
      'glib-compile-schemas',
      schema_dir,
    )
  endif

endif

# NOTE: Linux uses gnome postinstall script
