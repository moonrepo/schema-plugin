# Changelog

## 0.9.0

#### 🚀 Updates

- Updated to support proto v0.29 release.
- Added `globals.bin` setting, for customizing the bin used for installs/uninstalls.

## 0.8.0

#### 💥 Breaking

- Removed deprecated functions: `locate_bins`, `create_shims`

#### 🚀 Updates

- Updated to support proto v0.28 release.
- Updated to extism-pdk v1.

## 0.7.1

#### 🚀 Updates

- Added `resolve.version-pattern` and improved regex handling.
  - Now supports named captures: `major`, `minor`, `patch`, `pre`, `build`
  - Will construct the version from the above captures.
- Deprecated `resolve.git-tag-pattern` (use the above instead).

## 0.7.0

#### 🚀 Updates

- Updated to support proto v0.26 release.

#### ⚙️ Internal

- Updated dependencies.

## 0.6.0

#### 🚀 Updates

- Updated to support proto v0.24 release.

#### ⚙️ Internal

- Updated dependencies.

## 0.5.0

#### 🚀 Updates

- Added `install.no_bin` and `install.no_shim` fields.
- Updated to support proto v0.22 release.
- Deprecated undocumented `shim` setting.

#### ⚙️ Internal

- Updated dependencies.

## 0.4.1

#### 🐞 Fixes

- Potentially fixed a WASM memory issue.

## 0.4.0

#### 🚀 Updates

- Added `install.checksum_public_key` for defining the public key used to verify checksums.
- Added `metadata.self_upgrade_commands` for defining which sub-commands should be blocked for self-upgrades.
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
