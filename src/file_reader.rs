use futures::Future;
use image::buffer;
use js_sys::{Array, ArrayBuffer};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Document, Window, Response};

pub struct WasmFileReader {}


impl WasmFileReader {
    pub async fn read_file(path: &str) -> Vec<u8> {
        let window = web_sys::window().expect("");
        let response_js = JsFuture::from(window.fetch_with_str(path)).await.expect("Failed to fetch file");
        assert!(response_js.is_instance_of::<Response>());
        let response: Response = response_js.dyn_into().unwrap();
        
        let buffer_js = JsFuture::from(response.array_buffer().expect("I don't even know tbh")).await.expect("Could not read response body buffer");
        assert!(buffer_js.is_instance_of::<ArrayBuffer>());
        let buffer: ArrayBuffer = buffer_js.dyn_into().unwrap();
        let u8_buffer: js_sys::Uint8Array = js_sys::Uint8Array::new(&buffer);
        let mut buff_vec = vec![0; u8_buffer.length() as usize];
        u8_buffer.copy_to(&mut buff_vec[..]);
        buff_vec
    }
}