if is_dev
  # set dev icon if available
  # icon_name = '@0@.Dev.svg'.format(base_id)
  icon_name = '@0@.svg'.format(base_id)
else
  icon_name = '@0@.svg'.format(base_id)
endif

install_data(
  icon_name,
  install_dir: datadir / 'icons' / 'hicolor' / 'scalable' / 'apps',
  rename: '@0@.svg'.format(application_id),
)
