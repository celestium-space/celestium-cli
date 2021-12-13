import cv2
import json
import numpy
import random
import pathlib
import progressbar
from PIL import Image


def try_white_pixel(image, x, y):
    try:
        image.putpixel((x, y), (255, 255, 255, 255))
    except IndexError as _:
        return


def large_white_box(image, x, y):
    # Top line
    try_white_pixel(image, x - 2, y - 2)
    try_white_pixel(image, x - 1, y - 2)
    try_white_pixel(image, x, y - 2)
    try_white_pixel(image, x + 1, y - 2)
    try_white_pixel(image, x + 2, y - 2)

    # Left line
    try_white_pixel(image, x - 2, y - 1)
    try_white_pixel(image, x - 2, y)
    try_white_pixel(image, x - 2, y + 1)

    # Right line
    try_white_pixel(image, x + 2, y - 1)
    try_white_pixel(image, x + 2, y)
    try_white_pixel(image, x + 2, y + 1)

    # Bottom line
    try_white_pixel(image, x - 2, y + 2)
    try_white_pixel(image, x - 1, y + 2)
    try_white_pixel(image, x, y + 2)
    try_white_pixel(image, x + 1, y + 2)
    try_white_pixel(image, x + 2, y + 2)


def small_white_box(image, x, y):
    # Top line
    try_white_pixel(image, x - 1, y - 1)
    try_white_pixel(image, x, y - 1)
    try_white_pixel(image, x + 1, y - 1)

    # Sides
    try_white_pixel(image, x - 1, y)
    try_white_pixel(image, x + 1, y)

    # Bottom line
    try_white_pixel(image, x - 1, y + 1)
    try_white_pixel(image, x, y + 1)
    try_white_pixel(image, x + 1, y + 1)


seeds = []
with open("random.json", "r") as random_seeds:
    seeds = json.loads(random_seeds.read())

# seeds = [[random.randint(2, 1278), random.randint(2, 718)] for _ in seeds]

images = []
for file in pathlib.Path("images").iterdir():
    image = Image.open(file)
    orig_colors = []
    for x, y in seeds:
        orig_colors.append((x, y, image.getpixel((x, y))))
    for x, y in seeds:
        small_white_box(image, x, y)
    for x, y, orig_color in orig_colors:
        image.putpixel((x, y), orig_color)
    images.append(image)

images = images * 100
random.shuffle(images)

left = int(1280 / 2 - 100)
top = int(720 / 2 - 100)
right = left + 200
bottom = top + 200
images = [i.crop((left, top, right, bottom)) for i in images]

images[0].save(
    "out.gif",
    format="GIF",
    append_images=images[1:],
    save_all=True,
    loop=0,
)

# videodims = (1280, 720)
# fourcc = cv2.VideoWriter_fourcc(*"avc1")
# video = cv2.VideoWriter("test.mp4", fourcc, 60, videodims)
# for img in progressbar.progressbar(images):
#     imtemp = img.copy()
#     imcv = cv2.cvtColor(numpy.asarray(imtemp), cv2.COLOR_RGB2BGR)
#     video.write(imcv)

# # Deallocating memories taken for window creation
# cv2.destroyAllWindows()
# video.release()  # releasing the video generated
