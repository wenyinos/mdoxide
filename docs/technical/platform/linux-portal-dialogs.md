# Linux File Dialog Portal Requirements

This document explains the xdg-desktop-portal requirements for running Ferrite on Linux desktop environments, particularly minimal window managers and Wayland compositors.

## Overview

Ferrite uses the [rfd](https://github.com/PolyMeilex/rfd) crate for native file dialogs. On Linux, rfd uses the **xdg-desktop-portal** standard via the [ashpd](https://github.com/bilelmoussaoui/ashpd) backend to provide secure, sandbox-compatible file dialogs.

Some Linux desktop environments (particularly minimal window managers and Wayland compositors) require manual installation of xdg-desktop-portal packages for file dialogs to function.

## Affected Desktop Environments

The following Linux desktops typically require manual xdg-desktop-portal installation:

| Desktop Environment | Portal Package | Notes |
|---------------------|----------------|-------|
| **Hyprland** | `xdg-desktop-portal-hyprland` | Also requires `xdg-desktop-portal-wlr` as fallback |
| **Sway** | `xdg-desktop-portal-wlr` | wlroots-based portal implementation |
| **i3** | `xdg-desktop-portal-wlr` | Use with XDG_CURRENT_DESKTOP set |
| **bspwm** | `xdg-desktop-portal-wlr` | Minimal window manager |
| **dwm** | `xdg-desktop-portal-wlr` | Dynamic window manager |
| **awesomewm** | `xdg-desktop-portal-wlr` | Configurable window manager |
| **xmonad** | `xdg-desktop-portal-wlr` | Tiling window manager |
| **qtile** | `xdg-desktop-portal-wlr` | Python-based window manager |
| **river** | `xdg-desktop-portal-wlr` | Wayland compositor |
| **niri** | `xdg-desktop-portal-wlr` | Scrollable-tiling Wayland compositor |
| **COSMIC** | `xdg-desktop-portal-cosmic` | System76's Rust-based DE |
| **Wayfire** | `xdg-desktop-portal-wlr` | 3D Wayland compositor |
| **LabWC** | `xdg-desktop-portal-wlr` | Openbox clone for Wayland |

### Desktop Environments with Native Support

These desktop environments typically have built-in portal support and don't require manual installation:

| Desktop Environment | Status |
|---------------------|--------|
| **GNOME** | Built-in (uses xdg-desktop-portal-gnome) |
| **KDE Plasma** | Built-in (uses xdg-desktop-portal-kde) |
| **XFCE** | Usually has portal support |
| **MATE** | Usually has portal support |
| **Cinnamon** | Usually has portal support |
| **LXDE** | May need manual installation |
| **LXQt** | Usually has portal support |
| **Budgie** | Usually has portal support |

## Installation Instructions by Distro

### Arch Linux (and derivatives: Manjaro, EndeavourOS, Garuda)

```bash
sudo pacman -S xdg-desktop-portal xdg-desktop-portal-wlr

# For Hyprland specifically, also install:
sudo pacman -S xdg-desktop-portal-hyprland
```

### Debian / Ubuntu / Pop!_OS / Linux Mint

```bash
sudo apt install xdg-desktop-portal xdg-desktop-portal-wlr
```

### Fedora / Nobara

```bash
sudo dnf install xdg-desktop-portal xdg-desktop-portal-wlr
```

### openSUSE

```bash
sudo zypper install xdg-desktop-portal xdg-desktop-portal-wlr
```

## Configuration

### Hyprland Configuration

Add to your `~/.config/hypr/hyprland.conf`:

```
exec-once = /usr/lib/xdg-desktop-portal-hyprland
exec-once = /usr/lib/xdg-desktop-portal
```

Or if using systemd:

```
exec-once = systemctl --user import-environment PATH
exec-once = systemctl --user start xdg-desktop-portal-hyprland
exec-once = systemctl --user start xdg-desktop-portal
```

### Sway Configuration

Add to your Sway config:

```
exec dbus-update-activation-environment --systemd DISPLAY WAYLAND_DISPLAY SWAYSOCK XDG_CURRENT_DESKTOP
exec systemctl --user start xdg-desktop-portal-wlr
exec systemctl --user start xdg-desktop-portal
```

### Setting XDG_CURRENT_DESKTOP

For minimal window managers, ensure `XDG_CURRENT_DESKTOP` is set correctly:

```bash
# Add to ~/.profile or ~/.bashrc
export XDG_CURRENT_DESKTOP=sway  # or hyprland, i3, etc.
```

## How Ferrite Handles Portal Failures

### Detection

Ferrite detects the Linux desktop environment by checking the following environment variables:

1. `XDG_CURRENT_DESKTOP` - Primary detection method
2. `DESKTOP_SESSION` - Fallback detection

### Error Handling

When a file dialog fails on a Linux desktop that requires portals:

1. **Logging**: A warning is logged:  
   ```
   File dialog failed on Hyprland. This may indicate missing xdg-desktop-portal.
   ```

2. **Error Dialog**: An error modal is displayed with:
   - The detected desktop environment name
   - Installation instructions specific to the detected distro
   - A "Copy Install Command" button to copy the command to clipboard

3. **Distro-Specific Instructions**: The dialog shows the appropriate package manager command:
   - Arch: `pacman -S xdg-desktop-portal xdg-desktop-portal-hyprland`
   - Ubuntu: `apt install xdg-desktop-portal xdg-desktop-portal-wlr`
   - Fedora: `dnf install xdg-desktop-portal xdg-desktop-portal-wlr`

### Code Implementation

The portal error handling is implemented in:

- `src/files/dialogs.rs` - Detection functions and `DialogResult<T>` type
- `src/app/file_ops.rs` - Error handling in file operations
- `src/app/dialogs.rs` - Portal error dialog UI rendering

## Troubleshooting

### Dialog Still Fails After Installation

1. **Check portal service is running**:
   ```bash
   systemctl --user status xdg-desktop-portal
   systemctl --user status xdg-desktop-portal-wlr  # or -hyprland
   ```

2. **Restart portal services**:
   ```bash
   systemctl --user restart xdg-desktop-portal
   systemctl --user restart xdg-desktop-portal-wlr
   ```

3. **Verify environment variables**:
   ```bash
   echo $XDG_CURRENT_DESKTOP
   echo $WAYLAND_DISPLAY
   ```

### Flatpak-Specific Issues

If running Ferrite as a Flatpak:

1. Ensure the portal service is running on the host system
2. The portal dialog may need a navigable starting directory (Ferrite falls back to `$HOME`)
3. Check Flatpak permissions with: `flatpak info --show-permissions dev.ferrite.Ferrite`

## rfd Zenity Fallback

As of rfd 0.14+, the library does not automatically fall back to Zenity or KDialog when portals are unavailable. The implementation relies on the xdg-desktop-portal standard.

If you need an alternative file dialog without portals, consider:

1. Installing the required portal packages (recommended)
2. Using a desktop environment with native portal support

## Environment Variables Reference

| Variable | Purpose | Example Values |
|----------|---------|----------------|
| `XDG_CURRENT_DESKTOP` | Identifies the desktop environment | `hyprland`, `sway`, `gnome`, `kde` |
| `DESKTOP_SESSION` | Fallback desktop identification | `hyprland`, `sway`, `gnome` |
| `FLATPAK_ID` | Indicates Flatpak sandbox | `dev.ferrite.Ferrite` |
| `WAYLAND_DISPLAY` | Indicates Wayland session | `wayland-1` |
| `DISPLAY` | Indicates X11 session | `:0` |

## Related Documentation

- [Flatpak File Dialog Portal](./flatpak-file-dialog-portal.md) - Flatpak-specific portal handling
- [rfd crate documentation](https://docs.rs/rfd/)
- [xdg-desktop-portal specification](https://flatpak.github.io/xdg-desktop-portal/)
