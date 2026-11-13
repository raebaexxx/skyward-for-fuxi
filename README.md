![Skyward Recovery](assets/hero.jpg)

# Skyward for fuxi

English | [Русский](README.ru.md)

Skyward is a recovery written in Rust, from scratch. This fork is dedicated to
one phone: the **Xiaomi 13** (codename `fuxi`), and is maintained by
**RaeBaeXXX**. For the other devices the upstream project supports, see
[kirklin/skyward](https://github.com/kirklin/skyward).

![Skyward running on the Xiaomi 13](docs/overview.webp)

<p align="center">
  <img src="assets/home.png" width="30%" alt="Home">
  <img src="assets/settings-dark.png" width="30%" alt="Settings in dark mode">
  <img src="assets/time-zone.png" width="30%" alt="Time zone">
</p>

## Features

- Decrypt the data partition with the screen lock
- Install zips, system update packages and partition images, from internal storage, a USB drive or adb sideload
- Online system updates
- Backup and restore
- Wipe data and factory reset
- Mount partitions, repair and format filesystems
- File manager and terminal, with the list ordered by name, size or date
- Wireless transfer: internal storage in a browser on the network
- Download a file from an address into internal storage
- WLAN
- MTP file transfer, adb and fastbootd
- Reboot to system, recovery, fastboot, bootloader or EDL, and switch slots
- Light and dark themes; English and Russian

## Supported device

| Device | Codename | Tested system version |
| --- | --- | --- |
| Xiaomi 13 | `fuxi` | crDroid 17.0 (Android 17, 2026-11-13 build) |

Decryption was verified on this phone with a PIN screen lock: the data
partition is decrypted, the owner's credential-based files open, and internal
storage is readable.

The image is built on the Android 17 recovery of crDroid 17.0 for fuxi. That
recovery is kept as the `fuxi-a17-recovery.img` asset of this repository's
`base` release. An Android 16 stock recovery cannot be used for a crDroid 17
system: its libc++ cannot load the Android 17 vendor binaries the key service
is built from.

## What has been tested

Verified on the phone with crDroid 17.0: PIN decryption, internal storage,
WLAN, MTP, adb, installing a full system update package from internal storage
and keeping Skyward after it, installing a GApps zip with its own script,
wireless transfer to a browser, rebooting, and the About page.

Still to be verified on the phone: backup and restore, wipe, the file manager
and terminal, USB drives and `adb sideload`, the updater, dual boot, mounting,
repairing or formatting partitions, uploading through wireless transfer,
downloading by link, and the file order.

## Installing

Skyward needs an unlocked bootloader. Download the zip from this repository's
[Releases](https://github.com/raebaexxx/skyward-for-fuxi/releases), which are
published from tags named `fuxi-v*`. Check it against `SHA256SUMS.txt`, unzip
it, and flash the image inside to the recovery partition with the phone in
fastboot mode:

```sh
unzip skyward-fuxi-v1.6.zip
fastboot flash recovery skyward-fuxi-v1.6.img
fastboot reboot recovery
```

Use the name of the release you downloaded; the zip is named after its tag.

Afterwards a newer image, unzipped, installs from Skyward's own Install page.
With **Flash Skyward again** on, a system update or zip that writes over the
recovery partition gets Skyward back in it.

Without a system in super, Skyward still starts, with its screen and keys, adb,
`adb sideload` and fastbootd, enough to install one. Touch, WLAN and decryption
run from the installed system and come back with it.

adb in Skyward is a root shell that needs no authorization on the phone: anyone
with the phone and a cable can reach its storage while it is in recovery.

## Building

Skyward builds with the stable Rust that `rust-toolchain.toml` names, with its
`aarch64-linux-android` target, the Android NDK r29 and
[cargo-ndk](https://github.com/bbqsrc/cargo-ndk). The recovery service is built
for the phone, its second system's first program statically, and the packer for
the computer:

```sh
# SKYWARD_VERSION makes the build name the release it is for; a local build
# without it uses the workspace version, 1.6.0.
SKYWARD_VERSION=1.6 \
  cargo ndk -t arm64-v8a --platform 30 build --locked --release -p skyward
RUSTFLAGS="-C target-feature=+crt-static" \
  cargo ndk -t arm64-v8a --platform 30 build --locked --release \
  -p skyward-system-init --target-dir target/system-init
cargo build --locked --release -p skyward-pack
```

The base is the Android 17 recovery of the crDroid build installed on the
phone. This repository keeps the one the images here were tested on:

```sh
gh release download base --pattern fuxi-a17-recovery.img --dir stock
```

Or take the `recovery.img` out of a local crDroid
`out/target/product/fuxi/` build. Then build the image on top of it:

```sh
target/release/skyward-pack recovery \
  --stock stock/fuxi-a17-recovery.img \
  --binary target/aarch64-linux-android/release/skyward \
  --system-init target/system-init/aarch64-linux-android/release/system-init \
  --config devices/fuxi.toml \
  --font resources/fonts/MiSansVF.ttf \
  --mono resources/fonts/KirkMono-Regular.ttf \
  --mono-license resources/fonts/KirkMono-OFL.txt \
  --out target/skyward-fuxi.img
```

Tests and lints run on the computer; what runs only on the phone is linted for it:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo ndk -t arm64-v8a --platform 30 clippy -p skyward -p skyward-install -p skyward-systems -p skyward-system-init -- -D warnings
```

`skyward-preview` draws the pages on the computer, as PNG files, without a phone:
each page whole, as a long screenshot shows it, in the states it can be in, and the
screen at moments of touches and motion. `--lang` picks the language by a locale
such as `en` or `ru`:

```sh
cargo run --release -p skyward-preview -- target/preview --lang en
```

## Credits

Skyward was created by **Kirk Lin** and is licensed under the GPL-3.0-or-later.
This fuxi fork is maintained by **RaeBaeXXX**. The interface text is set in
Xiaomi's MiSans; the terminal uses the OFL-licensed Kirk Mono.

## License

[GPL-3.0-or-later](LICENSE)
