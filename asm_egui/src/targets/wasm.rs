use crate::app::EguiApp;
use eframe::CreationContext;
use eframe::epaint::text::{FontData, FontDefinitions};
use egui::{DroppedFileHandle, FontFamily};
use java_asm_server::rw_access::ReadAccess;
use java_asm_server::ui::AppContainer;
use java_asm_server::{AsmServer, ServerMut};
use js_sys::Uint8Array;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{self, Response};
use std::sync::Arc;

pub(crate) fn open_dropped_file(
    dropped_file: DroppedFileHandle,
    server: ServerMut,
    ui_app: AppContainer,
) {
    wasm_bindgen_futures::spawn_local(async move {
        let name = dropped_file
            .path()
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| "dropped-file".to_owned());
        match dropped_file.bytes_async().await {
            Ok(bytes) => AsmServer::smart_open(
                server,
                ReadAccess::from_raw(name, Arc::from(bytes.into_boxed_slice())),
                ui_app,
            ),
            Err(error) => log::error!("failed to read dropped file: {error}"),
        }
    });
}

const CJK_FONT_URL: &str =
    "https://cdn.jsdelivr.net/gh/notofonts/noto-cjk@Sans2.004/Sans/SubsetOTF/SC/NotoSansSC-Regular.otf";

pub(crate) fn configure_fonts(_context: &CreationContext) {}

fn set_cjk_font(context: &egui::Context, cjk_font: Vec<u8>) {
    let mut fonts = FontDefinitions::default();
    let cjk_font_name = "NotoSansSC-Regular";
    let cjk_font = FontData::from_owned(cjk_font);
    fonts.font_data.insert(cjk_font_name.into(), Arc::new(cjk_font));
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts.families.entry(family).or_default().push(cjk_font_name.into());
    }
    context.set_fonts(fonts);
}

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
                set_cjk_font(&context, font);
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
                    Ok(Box::new(EguiApp::new(cc)))
                }),
            ).await;
        match start_result {
            Ok(()) => hide_startup_loading(&document),
            Err(error) => log::error!("failed to start egui web runner: {error:?}"),
        }
    });
}
