#!/usr/bin/python3

import os
import shutil

cwd = os.getcwd()

command = "cargo r --release"
build_dir = "_build/"
docs_dir = "docs/images/"
cells = [
    "sram_16x16",
    "inv_dec",
    "nand2_dec",
    "sram_sp_cell",
    "colend",
    "corner",
    "rowend",
]

os.system(command)

os.chdir(build_dir)

for cell in cells:
    script = f"""
    magic -T sky130A -d XR -noconsole <<EOF
    load {cell}.mag
    select top cell
    expand
    findbox zoom
    select clear
    plot svg {cell}.svg
    quit -noprompt
    EOF
    """
    os.system(script)
    shutil.copyfile(f"{cell}.svg", f"{cwd}/{docs_dir}/{cell}.svg")