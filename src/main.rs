#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/* * Projeto: RustOps GUI
 * Autor: Danilo Ferreira Sousa
 * Descrição: Interface gráfica independente para rodar e interagir com modelos locais.
 */

mod app;
mod ollama;
mod storage;
mod utils;

use app::RustOpsApp;
use eframe;

// Função para carregar os pixels da imagem durante a compilação
fn carregar_icone() -> eframe::IconData {
    let image_bytes = include_bytes!("../icone.png");
    let image = image::load_from_memory(image_bytes)
        .expect("Falha ao carregar o ícone. Verifique se icone.png está na raiz do projeto.")
        .into_rgba8();
    
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    
    eframe::IconData {
        rgba,
        width,
        height,
    }
}

fn main() -> eframe::Result<()> {
    println!("=== INICIANDO INTERFACE GRÁFICA ===");

    let mut options = eframe::NativeOptions::default();
    options.icon_data = Some(carregar_icone());

    eframe::run_native(
        "RustOps GUI",
        options,
        Box::new(|_cc| Box::new(RustOpsApp::new())),
    )
}