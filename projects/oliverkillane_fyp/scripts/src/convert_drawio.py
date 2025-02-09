#!/usr/bin/env python
import copy
from pathlib import Path
import subprocess
from multiprocessing import Pool
import os


def get_drawio_files(from_path: Path) -> list[Path]:
    try:
        # Using the 'find' command to locate all .drawio files within 'diagrams' directories
        result = subprocess.run(
            ["find", from_path, "-type", "f", "-path", "*/diagrams/*.drawio"],
            capture_output=True,
            text=True,
            check=True,
        )
        # Splitting the output by newlines to get individual file paths
        drawio_files = result.stdout.strip().split("\n")
        return [Path(file) for file in drawio_files] if drawio_files[0] else []
    except subprocess.CalledProcessError as e:
        print(f"An error occurred: {e}")
        return []


def drawio_transform(old_path: Path):
    def drawio_image_out(x: Path) -> Path:
        parts = list(x.parts)
        parts[parts.index("diagrams")] = "_diagrams"
        return Path(*parts).with_suffix(".pdf")

    out_path = drawio_image_out(old_path)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    subprocess.check_output(
        ["drawio", old_path, "-x", "--format", "pdf", "-t", "--crop", "-o", out_path],
        stderr=subprocess.DEVNULL,
    )
    print(f"✅ transformed {out_path}")


def main() -> None:
    Pool().map(drawio_transform, get_drawio_files(Path(os.getcwd())))
