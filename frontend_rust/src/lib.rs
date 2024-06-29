use wasm_bindgen::prelude::*;
use web_sys::{HtmlElement, HtmlInputElement, HtmlCanvasElement, HtmlSelectElement, CanvasRenderingContext2d, console};
use reqwest::Client;
use serde::{Serialize, Deserialize};
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::Clamped;
use flate2::read::GzDecoder;
use std::io::Read;
use rmp_serde::decode::from_slice;
use js_sys;

#[derive(Serialize)]
struct InitData {
    env_name: String,
    seed: i32,
}

#[derive(Deserialize)]
struct RenderData {
    data: Vec<u8>,
    width: usize,
    height: usize,
}

const PYTHON_HOST: &str = "http://127.0.0.1:8000";
const WIDTH: u32 = 600;
const HEIGHT: u32 = 400;

async fn render(ctx: CanvasRenderingContext2d) -> Result<(), reqwest::Error> {
    ctx.clear_rect(0.0, 0.0, 600.0, 400.0);

    let client = Client::new();

    let window = web_sys::window().unwrap();
    let performance = window.performance().unwrap();
    let start = performance.now();

    let response = client.get(&format!("{}/render", PYTHON_HOST))
        .send()
        .await?;

    if !response.status().is_success() {
        web_sys::console::log_1(&JsValue::from_str(&format!("Failed to send render POST request: {}", response.status())));
        return Err(reqwest::Error::from(response.error_for_status().unwrap_err()));
    }

    let data = response.bytes().await?;

    let image_data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(Clamped(&data), WIDTH, HEIGHT).unwrap();
    ctx.put_image_data(&image_data, 0.0, 0.0).unwrap();
    let elapsed_total = performance.now() - start;
    web_sys::console::log_1(&JsValue::from_str(&format!("Total: {:?}", elapsed_total)));
    Ok(())
}

async fn init_env(env_name: String) -> Result<(), reqwest::Error> {
    let init_data = InitData { 
            env_name: env_name,
            seed: 42
        };

    let client = Client::new();
    let response = client.post(&format!("{}/init", PYTHON_HOST))
        .json(&init_data)
        .send()
        .await?;

    if !response.status().is_success() {
        web_sys::console::log_1(&JsValue::from_str(&format!("Failed to send init_env POST request: {}", response.status())));
        return Err(reqwest::Error::from(response.error_for_status().unwrap_err()));
    }

    Ok(())
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    
    let canvas = document.get_element_by_id("canvas").unwrap().dyn_into::<HtmlCanvasElement>().unwrap();
    let ctx = canvas.get_context("2d").unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>().unwrap();
    let select = document.get_element_by_id("init_env").unwrap().dyn_into::<HtmlSelectElement>().unwrap();
    let start_button = document.get_element_by_id("start").unwrap().dyn_into::<HtmlElement>().unwrap();

    let ctx_clone = ctx.clone();
    let select_clone = select.clone();

    let closure = Closure::wrap(Box::new(move || {
        let env_name = select_clone.value();
        let ctx_clone = ctx_clone.clone();
        spawn_local(async move {
            if let Err(err) = init_env(env_name).await {
                web_sys::console::log_1(&JsValue::from_str(&format!("Error: {:?}", err)));
            }
            if let Err(err) = render(ctx_clone).await {
                web_sys::console::log_1(&JsValue::from_str(&format!("Error: {:?}", err)));
            }
        });
    }) as Box<dyn Fn()>);

    start_button.set_onclick(Some(closure.as_ref().unchecked_ref()));
    closure.forget();
    Ok(())
}

