#!/bin/bash
set -e
mkdir -p assets
for size in 16 32 48 64 128 256; do
    rsvg-convert -w $size -h $size src/icons/app.svg -o assets/icon_${size}.png
done
convert assets/icon_16.png assets/icon_32.png assets/icon_48.png \
        assets/icon_64.png assets/icon_128.png assets/icon_256.png \
        assets/icon.ico
cp assets/icon_256.png assets/icon.png
echo "Done."
ls -la assets/

