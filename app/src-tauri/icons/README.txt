Generated using https://icon.kitchen/ (base, 0% padding)

For reference:
Windows (.ico): Requires an ICO file containing multiple sizes (16x16, 32x32, 48x48, 256x256)
macOS (.icns): Requires an ICNS file with multiple sizes (16x16, 32x32, 64x64, 128x128, 256x256, 512x512, 1024x1024)
Linux: Typically needs PNG files in sizes: 32x32, 128x128, and 128x128@2x (256x256)

For web: use the following as reference <head>:

    <link rel="icon" href="/favicon.ico" sizes="any">
    <link rel="apple-touch-icon" href="/apple-touch-icon.png">

Add this to `tauri.conf.json`:

    ...
    "icon": [
      "icons/AppIcon.icns",
      "icons/icon-192-maskable.png",
      "icons/icon-512-maskable.png",
      "icons/favicon.ico"
    ]
    ...
