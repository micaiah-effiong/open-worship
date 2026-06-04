cargo = find_program('cargo')
cargo_options = ['--manifest-path', meson.project_source_root() / 'Cargo.toml']
cargo_options += ['--target-dir', meson.project_build_root() / 'target']

if not is_dev
  cargo_options += ['--locked']
  cargo_options += ['--profile']
  cargo_options += ['production']
endif

env = {
  'CARGO_HOME': meson.project_build_root() / 'cargo-home',
  'APP_ID': application_id,
  'RESOURCE_FILE': meson.project_build_root() / 'data' / 'resources' / 'resources.gresource',
  'RESOURCE_SCHEMA': meson.project_build_root() / 'data' / '@0@.gschema.xml'.format(application_id),
  'GSETTINGS_SCHEMA_DIR': meson.project_build_root() / 'data',
}

if is_dev
  glib_compile_bin = find_program('glib-compile-schemas')
  glib_compile = custom_target(
    'glib-compile',
    build_by_default: true,
    build_always_stale: true,
    console: true,
    install: false,
    command: [
      glib_compile_bin,
      meson.project_build_root() / 'data',
      '&&',
      'touch',
      '@OUTPUT@',
    ],
    output: 'glib_compile.stamp',
  )
endif

if host_machine.system() == 'windows'
  src = meson.project_build_root() / 'target' / rust_target / meson.project_name()
  full_cmd = cargo.full_path() + ' build ' + ' '.join(cargo_options) + ' && cp ' + src + ' "$1"'

  custom_target(
    'cargo-build',
    build_by_default: true,
    build_always_stale: true,
    console: true,
    install: true,
    install_dir: bindir,
    depends: [resources],
    env: env,
    command: [
      'sh',
      '-c', full_cmd,
      '--', '@OUTPUT@',
    ],
    output: meson.project_name() + '.exe',
  )

  meson.add_install_script(
    'bash',
    meson.project_source_root() / 'build-aux' / 'copy-win-dll.sh',
    src + '.exe',
    bindir,
  )

  # todo: copy win dll
else
  custom_target(
    'cargo-build',
    build_by_default: true,
    build_always_stale: true,
    console: true,
    install: true,
    install_dir: bindir,
    depends: [resources],
    env: env,
    command: [
      cargo,
      'build',
      cargo_options,
      '&&',
      'cp',
      meson.project_build_root() / 'target' / rust_target / meson.project_name(),
      '@OUTPUT@',
    ],
    output: meson.project_name(),
  )
endif

if is_dev
  config_trg = custom_target(
    'cargo-config',
    build_by_default: true,
    build_always_stale: true,
    console: true,
    install: false,
    command: [
      'sh',
      '-c', 'cp "@0@" "@1@/.cargo/config.toml"'.format(
        meson.project_build_root() / 'config.toml',
        meson.project_source_root(),
      ),
    ],
    output: 'cargo-config.stamp',
  )

  run_target(
    'cargo-run',
    depends: [resources, glib_compile, config_trg],
    env: env,
    command: [
      cargo,
      'run',
      cargo_options,
    ],
  )
endif
