import os
import shutil
from pathlib import Path

images = list(Path("video").iterdir())
max_image = int(sorted(images, key=lambda i: int(i.stem))[-1].stem)

for i in range(max_image + 1):
    image = Path(f"video/{i:0>10}.png")
    if not image.exists():
        print(f"OH NO! {image}")
        shutil.copy(Path(f"video/{i-1:0>10}.png"), image)

os.system("ffmpeg -r 120 -i video/%10d.png -c:v libx265 canvas.mp4 -y")
