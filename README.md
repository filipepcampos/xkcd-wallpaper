# xkcd-wallpaper

[![MIT License](https://img.shields.io/github/license/filipepcampos/xkcd-wallpaper)](LICENSE)
![Build status](https://img.shields.io/github/actions/workflow/status/filipepcampos/xkcd-wallpaper/rust.yml)

Turn any XKCD comic into a wallpaper.

Fetches a comic (latest or by number), lays it on a canvas, replaces the background with your chosen hex colour, and saves the finished image -- ready to set as your desktop background. This can be combined with a startup script to replace your wallpaper daily.


## Install

### Stable release (planned)
> [!CAUTION]
> This is planned for the future, as of this moment, this is still not possible

```bash
cargo install xkcd-wallpaper
```

### From source

```sh
git clone https://github.com/filipepcampos/xkcd-wallpaper.git
cd xkcd-wallpaper
cargo install --path .
```

## Usage

Print the full CLI reference:

```
xkcd-wallpaper --help
```

Generate a 2560 × 1440 wallpaper from comic 3084 with a dark‑green background:

```
xkcd-wallpaper \
  --width 2560 --height 1440 \
  --bg "#1F241F" \
  --fg light \
  --comic 3084
```

## Example output

Original comic             |  Wallpaper |
:-------------------------:|:-------------------------:|
![Original commit](example/comic_1.png)  |  ![Wallpaper](example/wallpaper_1.png) |
![Original commit](example/comic_2.png)  |  ![Wallpaper](example/wallpaper_2.png) |
![Original commit](example/comic_3.png)  |  ![Wallpaper](example/wallpaper_3.png) |


## License

Distributed under the [MIT License](LICENSE). 

[XKCD comics](https://xkcd.com) © Randall Munroe; used here under the terms of the [XKCD license](https://xkcd.com/license.html).
