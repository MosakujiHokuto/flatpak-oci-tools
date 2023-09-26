# flatpak-oci-tools

**Still in early development, interface may change**

Tools for constructing flatpak from oci images

## Build

```
$ cargo build
```

## Usage

TODO: Details usage

Import a container image and create a flatpak runtime from it:

```
# flatpak-oci-tools import-container <container> <repo>
```

Fetch a container image from obs:

```
# flatpak-oci-tools obs-fetch --username <username> --password <password> <project> <repository> <architecture> <package>
```

**WIP** Pulling an image from an OCI registry:

_This command is not fully implemented yet and is subject to changes_

```
# flatpak-oci-tools pull <container name>
```

Currently this commands pulls container from OBS registry [https://registry.opensuse.org/] under `home:yudaike:flatpak-oci-container` project by default. See `flatpak-oci-tools pull --help` for details.

examples:
```
# flatpak-oci-tools pull firefox
```

```
# flatpak-oci-tools pull gedit
```
