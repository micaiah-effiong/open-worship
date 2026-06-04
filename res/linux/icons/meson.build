if is_dev
  # set dev icon if available
  # icon_name = '@0@.Devel.svg'.format(base_id)
  icon_name = '@0@.svg'.format(base_id)
else
  icon_name = '@0@.svg'.format(base_id)
endif

configure_file(
  input: icon_name,
  output: '@0@.svg'.format(application_id),
  copy: true,
  install: true,
  install_dir: datadir / 'icons' / 'hicolor' / 'scalable' / 'apps',
)
