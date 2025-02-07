## Windows

- Install rust from [the rust site](https://www.rust-lang.org/tools/install) then run

  ```sh
  rustup default stable-msvc
  ```

- Install MSYS2 see [instructions](https://www.msys2.org/)
  - after installing run
  ```sh
  pacman -S mingw-w64-ucrt-x86_64-gcc
  ```
- Install GVSbuild

  ```sh
  py -3.13 -m pip install --user pipx
  py -3.13 -m pipx ensurepath
  pipx install gvsbuild
  ```

  read more on [gvsbuild](https://github.com/wingtk/gvsbuild)

- Build gtk4 with gvsbuild

```sh
gvsbuild build gtk4
```

- Add gtk to environment variable
  `C:\gtk-build\gtk\x64\release\bin`

- Build librsvg and libadwaita

  ```sh
  gvsbuild build librsvg libadwaita
  ```

  > If you encounter "A required privilege is not held by the client. errno = 1314"
  > while building libsvg, enable the for developer feature and retry the command.
  > Windows settings (Settings -> Update & security -> For developers)

#### Note

> If you encounter an error during this setup and cannot processed, create an
> issue with details on the error and lable it as `question`

## MacOS

- Install rust follow [instructions](https://rustup.rs/) on the rust site

- Using hombrew setup gtk

  ```sh
  #install gtk4
  brew install gtk4

  #install librsvg and libadwaita
  brew install librsvg libadwaita
  ```

## Linux

- Using Debian and its derivatives
  ```sh
  sudo apt install libgtk-4-dev build-essential
  ```
- For other distros see [instructions](https://gtk-rs.org/gtk4-rs/stable/latest/book/installation_linux.html)

> #### Note:
>
> For more information on setting up gtk4 see the [installation docs](https://gtk-rs.org/gtk4-rs/stable/latest/book/installation.html)
