import os
import shutil
from pathlib import Path

# build with 
# wasm-pack build --release --target web
# if os.path.exists("build"):
#     shutil.rmtree("build")
# os.mkdir("build")

# shutil.copy("../wasm-hashlife/pkg/wasm_hashlife_bg.wasm","build/wasm_hashlife_bg.wasm");
# shutil.copy("../wasm-hashlife/pkg/wasm_hashlife.js","build/wasm_hashlife_bg.js");

for subpath in Path("examples").iterdir():
    print(subpath)
    # shutil.copy(subpath, Path("build") / os.path.basename(subpath))