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

Pulling an image from an OCI registry:

```
# flatpak-oci-tools pull <container name>
```
```
$ flatpak-oci-tools --user pull <container name>
```

Currently this commands pulls container from OBS registry [https://registry.opensuse.org/] under `home:yudaike:flatpak-oci-container` project by default. See `flatpak-oci-tools pull --help` for details.

Pulling an image from an OCI registry, and install it into system:
```
# flatpak-oci-tools install <container name>
```

User mode install variant:

```
$ flatpak-oci-tools --user install <container name>
```

examples:
```
# flatpak-oci-tools pull firefox
```

```
# flatpak-oci-tools pull gedit
```

## Example

```
$ flatpak-oci-tools --user install firefox
$ flatpak run org.openSUSE.App.MozillaFirefox
```

```
$ flatpak-oci-tools --user install gedit
$ flatpak run org.openSUSE.App.gedit
```
