console.log("hello, from trellis view.");

console.log("importing wasm code...");


// import * as wasm from "./pkg/trellis_view_wasm.js";
// console.log("maybe we imported wasm?");
// console.log("wasm: ", wasm);

import init, * as tv from "./pkg/trellis_view_wasm.js";

var app = null;
var canvas = null;
var context = null;

init().then(inited => {
    console.log("imported package");    
    // tv.init_rust();
    // console.log("hello = ", tv.hello_world());

    
    
    // setting up canvas

    var canvas = window.document.getElementById("trellis_canvas");
    var context = canvas.getContext("2d");

    app = tv.make_app();
    tv.paint_graph(app, context);
   
})


/*
const importObject = Object();
importObject.alert = function(message) {
    window.alert(message);
}

console.log("importObject = ", typeof(importObject));

fetch('./pkg/trellis_view_wasm_bg.wasm').then(response  => {
    console.log("got wasm code: " + response.statusText);
    return response.arrayBuffer();
})
.then(bytes => {
    console.log("got arrayBuffer: ", bytes.byteLength);
    return WebAssembly.instantiate(bytes, importObject);
})
.then(trellis_view => {
    console.log("got wasm: ");

    var x = trellis_view.instance.exports.hello_world();
    console.log("x = ", x);
})
.catch(e => {
    console.log("failed to load: " + e);
})
*/


/*

WebAssembly.instantiateStreaming(fetch('trellis_view_wasm.wasm'), importObject)
.then(obj => {
  // Call an exported function:
  obj.instance.exports.hello_world();

  // or access the buffer contents of an exported memory:
//   var i32 = new Uint32Array(obj.instance.exports.memory.buffer);

  // or access the elements of an exported table:
//   var table = obj.instance.exports.table;
//   console.log(table.get(0)());
})
.catch(e => {
    console.log("failed to load: " + e);
})
*/
