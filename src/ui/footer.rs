use eframe::egui;
use crate::app::RustOpsApp;
use crate::storage::ChatMessage;
use std::sync::mpsc::channel;
use std::thread;
use crate::ollama;

pub fn desenhar_rodape(app: &mut RustOpsApp, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("rodape").show(ctx, |ui| {
        ui.add_space(10.0);

        // horizontal_top mantém o botão alinhado ao topo enquando o input cresce
        ui.horizontal_top(|ui| {

            let largura_disponivel = ui.available_width() - 80.0;
            let altura_maxima_input = 100.0;

            // 1. Área de rolagem para o texto (limita o crescimento vertical)
            egui::ScrollArea::vertical()
                .max_height(altura_maxima_input)
                .id_salt("scroll_input_usuario")
                .show(ui, |ui| {
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut app.user_input)
                            .hint_text("Digite sua mensagem aqui... (Shift + Enter para nova linha)")
                            .desired_width(largura_disponivel)
                            .desired_rows(2)
                            .lock_focus(true)
                    );

                    // 2. Lógica de detecção do teclado dentro da closure
                    if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift) {
                        app.requisitou_envio = true;
                    }
                });

                // Botão com tamanho fixo
                let button = ui.add_sized([70.0, 35.0], egui::Button::new("Enviar"));

                // 3. Processamento do Envio (seka por clique ou por teclado)
                if (button.clicked() || app.requisitou_envio) && !app.user_input.trim().is_empty() && !app.is_processing {
                    app.requisitou_envio = false; // Reset imediato da flag
                

                // --- CÓDIGO DE ENVIO (OLLAMA) ---
                let historico_completo = {
                    let sessao_atual = app.db.get_sessao_ativa_mut();
                    
                    // Adiciona mensagem do usuário
                    sessao_atual.mensagens.push(ChatMessage {
                        role: "user".to_string(),
                        content: app.user_input.trim().to_string(),
                    });

                    let historico_para_api = sessao_atual.mensagens.clone();

                    // Prepara o balão da resposta da IA
                    sessao_atual.mensagens.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: "".to_string(),
                    });

                    historico_para_api
                };
                app.db.salvar();
                app.user_input.clear();
                app.is_processing = true;

                let (tx, rx) = channel();
                app.receptor_de_texto = Some(rx);

                thread::spawn(move || {
                    ollama::send_to_ollama_chat(historico_completo, tx);
                });
                // --- FIM DO CÓDIGO DE ENVIO ---
            }
        });
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            // 1. Status de Hardware (Esquerda)
            ui.spacing_mut().item_spacing.x = 15.0; // Espaço entre os itens


            // CPU com cor dinâmica
            let cpu_cor = if app.cpu_usage > 70.0 {
                egui::Color32::from_rgb(255,100,100) // Vermelho se estiver alto
            } else {
                egui::Color32::DARK_GRAY
            };

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("CPU:")
                    .small()
                    .color(cpu_cor));
                ui.label(
                    egui::RichText::new(format!("{:.1}%", app.cpu_usage))
                    .small()
                    .color(cpu_cor)
                );
            });

            // RAM
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("RAM:")
                    .small()
                    .color(egui::Color32::DARK_GRAY));
                ui.label(
                    egui::RichText::new(format!("{:.1} GB", app.ram_usage))
                    .small()
                    .color(egui::Color32::DARK_GRAY)
                );
            });

            // 2. Assinatura e Versão (Direita)
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                        .small()
                        .color(egui::Color32::from_rgb(80,80,80))
                );

                ui.label(
                    egui::RichText::new(format!("| {} |", env!("CARGO_PKG_AUTHORS")))
                    .small()
                    .color(egui::Color32::DARK_GRAY)
                );
            });
        });
        ui.add_space(6.0);
    });
}