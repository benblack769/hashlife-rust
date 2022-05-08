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
var xstart = 0;
var ystart = 0;
var cellSizeSelect = document.getElementById("cell-size-select");
var zoomSelect = document.getElementById("zoom-select");
var brightnessSelect = document.getElementById("brightness-select");
brightnessSelect.addEventListener('change',render);
cellSizeSelect.addEventListener('change',render);
zoomSelect.addEventListener('change',render);

function cellSize(){
    return Math.pow(2,cellSizeSelect.value)
}
function zoomScale(){
    return Math.pow(2, zoomSelect.value)/cellSize() 
}
function brightness(){
    return Math.pow(2,0.25*brightnessSelect.value)
}
function roundToCell(size){
    return Math.floor(size/cellSize())*cellSize()
}
var cellCountDisplay = document.getElementById("cell-count");
var hashCountDisplay = document.getElementById("hash-count");
function render(){
    // console.log(tree.hash_count());
    // console.log(tree.num_live_cells());
    var map = tree.make_grayscale_map(xstart,ystart,xsize/cellSize(), ysize/cellSize(),cellSize(),zoomSelect.value,brightness());
    // console.log(map);
    var clamped_data = new Uint8ClampedArray(map);
    // console.log(clamped_data);
    var img_data =new ImageData(clamped_data,roundToCell(xsize),roundToCell(ysize));
    // console.log(img_data);
    canvas.width = xsize;
    canvas.height = ysize;
    const canvasContext = canvas.getContext("2d");
    canvasContext.imageSmoothingEnabled = false;
    canvasContext.putImageData(img_data, 0, 0);
    cellCountDisplay.innerText = tree.num_live_cells();
    hashCountDisplay.innerText = tree.hash_count();
}
window.onresize = function(event) {
    xsize = window.innerWidth - 50;
    ysize = window.innerHeight - 50;
    canvas.width = xsize;
    canvas.height = ysize;
    // isZooming();
};
var speedSelect = document.getElementById("speed-select");
var fpsSelect = document.getElementById("fps-select");
var garbageSelect = document.getElementById("garbage-select");
const renderLoop = () => {
    render();
    if (speedSelect.value > 0){
        tree.step_forward(Math.pow(2,speedSelect.value));
    }
    if (tree.hash_count() > 0.9*garbageSelect){
        tree.garbage_collect();
    }
    console.log(window.screen.availWidth / document.documentElement.clientWidth);
    setTimeout(renderLoop, 1000/Math.pow(2,fpsSelect.value/2.));
};
renderLoop();

function handle_wheel(event){
    // console.log(event);
    if(event.ctrlKey){
        console.log("zoomed! ",event.deltaY);
    }
    else{
        xstart += event.deltaX*zoomScale();
        ystart += event.deltaY*zoomScale();
    }
    render();
    event.stopPropagation();
}
canvas.addEventListener("wheel", handle_wheel);

var inputFileLoader = document.getElementById("rle-file-input");
function handleFileUpload() {
    var file = inputFileLoader.files[0];
    const reader = new FileReader();
    reader.onload = function(){
        tree = TreeDataWrapper.make_from_rle(reader.result);
    }
    reader.readAsText(file);
}
inputFileLoader.addEventListener("change", handleFileUpload, false);
