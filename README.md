# pokemonItor

A lightweight system tray application that monitors [ltratcliff.com](https://ltratcliff.com) for the presence of an "About" page link. When detected, it shows a toast notification.

## What it does

- Sits in the Windows system tray with a green icon
- Checks ltratcliff.com every 30 minutes for an `<a>` tag with the text "About"
- Shows a Windows toast notification when the About link is found
- Shows an error notification if the site check fails
- Right-click the tray icon and select **Quit** to exit

## Requirements

- Windows 10 or later
- [Rust toolchain](https://rustup.rs/) (for building from source)

## Building

```
cargo build --release
```

The compiled binary will be at `target\release\pokemonItor.exe`.

## Running

```
.\target\release\pokemonItor.exe
```

The application will start minimized to the system tray. The first site check runs immediately on startup, then repeats every 30 minutes.

## Notifications

When the About link is detected, a toast notification appears and stays visible for 10 seconds. Error notifications also display if the site is unreachable.

Make sure notifications for the application are enabled in **Settings > System > Notifications**.
