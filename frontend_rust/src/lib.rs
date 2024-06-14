use wasm_bindgen::prelude::*;
use web_sys::{HtmlElement, HtmlInputElement, HtmlCanvasElement,CanvasRenderingContext2d, console};
use reqwest::Client;
use serde::{Serialize, Deserialize};
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::Clamped;
use flate2::read::GzDecoder;
use std::io::Read;

#[derive(Serialize)]
struct InputData {
    input: i32,
}

#[derive(Deserialize, Serialize)]
struct ResponseData {
    output: Vec<Vec<Vec<u8>>>,
}

#[derive(Deserialize)]
struct ConvertData {
    data: Vec<u8>,
    width: usize,
    height: usize,
}

async fn send_post_request(input_value: i32, ctx: CanvasRenderingContext2d) -> Result<(), reqwest::Error> {
    const PYTHON_HOST: &str = "http://127.0.0.1:8000";

    ctx.clear_rect(0.0, 0.0, 600.0, 400.0);

    let client = Client::new();
    let input_data = InputData { input: input_value };

    let window = web_sys::window().unwrap();
    let performance = window.performance().unwrap();
    let start = performance.now();

    let response = client.post(&format!("{}/api/", PYTHON_HOST))
        .json(&input_data)
        .send()
        .await?;

    if !response.status().is_success() {
        web_sys::console::log_1(&JsValue::from_str(&format!("Failed to send POST request: {}", response.status())));
        return Err(reqwest::Error::from(response.error_for_status().unwrap_err()));
    }

    let data = response.json::<ConvertData>().await?;

    let elapsed_convert = performance.now() - start;

    let image_data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(Clamped(&data.data), data.width as u32, data.height as u32).unwrap();
    ctx.put_image_data(&image_data, 0.0, 0.0).unwrap();
    let elapsed_draw = performance.now() - start - elapsed_convert;
    let elapsed_total = performance.now() - start;
    web_sys::console::log_1(&JsValue::from_str(&format!("Convert: {:?}, Draw: {:?}, Total: {:?}", elapsed_convert, elapsed_draw, elapsed_total)));
    Ok(())
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let input = document.get_element_by_id("input").unwrap().dyn_into::<HtmlInputElement>().unwrap();
    let output = document.get_element_by_id("output").unwrap().dyn_into::<HtmlElement>().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap().dyn_into::<HtmlCanvasElement>().unwrap();
    let ctx = canvas.get_context("2d").unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>().unwrap();

    let input_clone = input.clone();
    let output_clone = output.clone();
    let ctx_clone = ctx.clone();

    let closure = Closure::wrap(Box::new(move || {
        let input_value = input_clone.value().parse::<i32>();
        match input_value {
            Ok(value) => {
                //let input_value_clone = input_value.clone();
                //let output_clone = output_clone.clone();
                let ctx_clone = ctx_clone.clone();
                spawn_local(async move {
                    if let Err(err) = send_post_request(value, ctx_clone).await {
                        web_sys::console::log_1(&JsValue::from_str(&format!("Error: {:?}", err)));
                    }
                });
            }
            Err(_) => {
                output_clone.set_inner_text("Please enter a valid number.");
            }
        }
    }) as Box<dyn Fn()>);

    let button = document.get_element_by_id("button").unwrap().dyn_into::<HtmlElement>().unwrap();
    button.set_onclick(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
    Ok(())
}

