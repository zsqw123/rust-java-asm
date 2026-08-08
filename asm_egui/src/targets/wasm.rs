use crate::app::EguiApp;
use crate::font::inject_cjk_font;
use js_sys::Uint8Array;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{self, Response};

const CJK_FONT_URL: &str =
    "https://cdn.jsdelivr.net/gh/notofonts/noto-cjk@523d033/Sans/OTF/SimplifiedChinese/NotoSansCJKsc-Regular.otf";

async fn load_web_font() -> Result<Vec<u8>, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let response = JsFuture::from(window.fetch_with_str(CJK_FONT_URL)).await?;
    let response: Response = response.dyn_into()?;
    if !response.ok() {
        return Err(JsValue::from_str(&format!(
            "font request failed with HTTP {}",
            response.status()
        )));
    }
    let bytes = JsFuture::from(response.array_buffer()?).await?;
    Ok(Uint8Array::new(&bytes).to_vec())
}

fn hide_startup_loading(document: &web_sys::Document) {
    let Some(loader) = document.get_element_by_id("startup-loading") else {
        return;
    };
    let _ = loader.set_attribute("hidden", "");
}

fn load_cjk_font_in_background(context: egui::Context) {
    wasm_bindgen_futures::spawn_local(async move {
        match load_web_font().await {
            Ok(font) => {
                inject_cjk_font(&context, font);
                context.request_repaint();
            }
            Err(error) => log::warn!("failed to load browser CJK font: {error:?}"),
        }
    });
}

pub fn main() {
    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("canvas")
            .expect("Failed to find canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(move |cc| {
                    load_cjk_font_in_background(cc.egui_ctx.clone());
                    Ok(Box::new(EguiApp::new(cc, None)))
                }),
            ).await;
        match start_result {
            Ok(()) => hide_startup_loading(&document),
            Err(error) => log::error!("failed to start egui web runner: {error:?}"),
        }
    });
}
