import {paniky, set_panic_hook_js, ExampleStruct, TreeDataWrapper} from "wasm-game-of-life";

set_panic_hook_js();

const RLE_STR = (
    "x = 12, y = 8, rule = B3/S23\n" +
    "5bob2o$4bo6bo$3b2o3bo2bo$2obo5b2o$2obo5b2o$3b2o3bo2bo$4bo6bo$5bob2o!\n"
);
var tree = TreeDataWrapper.make_from_rle(RLE_STR);
var canvas = document.getElementById("game-of-life-canvas");
var xsize = window.innerWidth - 50;
var ysize = window.innerHeight - 50;
function render(){
    console.log(tree.hash_count());
    console.log(tree.num_live_cells());
    var map = tree.make_grayscale_map(0,0,xsize, ysize,0,1.);
    // console.log(map);
    var clamped_data = new Uint8ClampedArray(map);
    // console.log(clamped_data);
    var img_data =new ImageData(clamped_data,xsize,ysize);
    // console.log(img_data);
    canvas.width = xsize;
    canvas.height = ysize;
    const canvasContext = canvas.getContext("2d");
    canvasContext.putImageData(img_data,0,0);
}
window.onresize = function(event) {
    xsize = window.innerWidth - 50;
    ysize = window.innerHeight - 50;
    canvas.width = xsize;
    canvas.height = ysize;
};
const renderLoop = () => {
    render();
    tree.step_forward(1);

    setTimeout(renderLoop, 100);
};
renderLoop();
// requestAnimationFrame(renderLoop);
