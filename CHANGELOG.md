# Changelog

## 0.4.0

#### 🚀 Updates

- Added `install.checksum_public_key` for defining the public key used to verify checksums.
- Updated to support proto v0.20 release.

#### ⚙️ Internal

- Updated dependencies.

## 0.3.3

#### 🐞 Fixes

- Fixed `archive-prefix` not being interpolated.

#### ⚙️ Internal

- Updated dependencies.

## 0.3.2

#### ⚙️ Internal

- Updated dependencies.

## 0.3.1

#### 🐞 Fixes

- Fixed an invalid regex pattern.

## 0.3.0

#### 🚀 Breaking

- We updated the schema internally to be represented as JSON instead of TOML, which may cause breakages depending on a version mismatch between proto and the plugin.

#### 🐞 Fixes

- Fixed version parsing from tags to be more accurate. Will now properly include prerelease/build metadata.

## 0.2.0

#### 🚀 Updates

- Added support for installing canary releases with the `install.checksum_url_canary` and `install.download_url_canary` settings.
- Updated to support proto v0.17 release.

## 0.1.1

#### 🚀 Updates

- Updated to support proto v0.16 release.

## 0.1.0

#### 🚀 Updates

- Added support for `install_global` and `uninstall_global`.
- Updated to support proto v0.15 release.

## 0.0.1

#### 🎉 Release

- Initial release!
