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
# flatpak-oci-tools fetch --username <username> --password <password> <project> <repository> <architecture> <package>
```
