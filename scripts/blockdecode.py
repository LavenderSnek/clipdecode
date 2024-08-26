import os
import numpy as np
from PIL import Image
from dataclasses import dataclass
import pathlib

# Script to decode a list of blocks into the raw images


@dataclass
class ClipImg:
    transparency: Image
    color: Image

    def merged(self) -> Image:
        img = self.color.copy()
        img.putalpha(self.transparency)
        return img

    def save_to_dir(self, dp: str):
        pathlib.Path(dp).mkdir(parents=True, exist_ok=True)
        self.transparency.save(pathlib.Path(dp) / 'transparency.png')
        self.color.save(pathlib.Path(dp) / 'color.png')
        self.merged().save(pathlib.Path(dp) / 'merged.png')


def decode_offscreen_chunk_blocks(width: int, height: int, filepaths: list[str]):
    transparency_final = Image.new('L', (width, height))
    color_final = Image.new('RGBA', (width, height))

    col = 0
    row = 0

    for fp in filepaths:
        with open(fp, 'rb') as f:
            # transparency
            transparency_bytes = f.read()
            transparency = Image.frombytes('L', (256, 256), transparency_bytes)
            transparency_final.paste(transparency, (col, row))

            # color
            f.seek(0x10000)
            color_bytes = f.read()
            color = Image.frombytes('RGBA', (256, 256), color_bytes)
            arr_color = np.array(color)
            arr_color = arr_color[:, :, [2, 1, 0]]  # convert from BGR -> RGB
            color = Image.fromarray(arr_color)

            color_final.paste(color, (col, row))

            col += 256
            if col >= width:
                col = 0
                row += 256

    return ClipImg(transparency_final, color_final)


def decode_offscreen_chunk_from_dir(width: int, height: int, dp: str):
    paths = [f"{dp}/{x}" for x in os.listdir(dp) if x.startswith('block')]
    paths.sort()

    return decode_offscreen_chunk_blocks(width, height, paths)


if __name__ == '__main__':
    import sys

    if len(sys.argv) != 5:
        print("usage: blockdecode.py [w] [h] [in_dir] [out_dir]")

    w = int(sys.argv[1])
    h = int(sys.argv[2])
    in_dir = sys.argv[3]
    out_dir = sys.argv[4]

    i = decode_offscreen_chunk_from_dir(w, h, in_dir)
    i.save_to_dir(out_dir)



