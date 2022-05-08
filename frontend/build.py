import os
import shutil

# build with 
# wasm-pack build --release --target web
shutil.copy("../wasm-hashlife/pkg/wasm_hashlife_bg.wasm","wasm_hashlife_bg.wasm");
shutil.copy("../wasm-hashlife/pkg/wasm_hashlife.js","wasm_hashlife_bg.js");
