use eframe::egui;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use crate::storage::{AppDatabase, ChatMessage};
use crate::ollama;
use crate::utils;

pub struct RustOpsApp {
    user_input: String,
    db: AppDatabase,
    receptor_de_texto: Option<Receiver<String>>,
    is_processing: bool,
    
    // Controle de edição do título
    editando_titulo: bool,
    novo_titulo_temp: String,

    // Variáveis para a tela de carregamento
    is_initialized: bool,
    startup_receiver: Option<Receiver<String>>,
    startup_status_text: String,
}

// =========================================================
// INICIALIZAÇÃO E THREAD DE CARREGAMENTO
// =========================================================
impl RustOpsApp {
    pub fn new() -> Self {
        let (tx, rx) = channel::<String>();

        // Thread rodando em segundo plano para não travar a interface
        thread::spawn(move || {
            // 1. Verifica/Instala Ollama
            let _ = tx.send("Verificando motor de IA (Ollama)...".to_string());
            if !utils::is_ollama_installed() {
                let _ = tx.send("Instalando Ollama... A janela de senha do sistema pode aparecer.".to_string());
                let _ = utils::instalar_ollama(); // Usa a função multiplataforma que criamos
            }

            // 2. Inicia o serviço
            let _ = tx.send("Iniciando serviço local do motor de IA...".to_string());
            if !utils::ollama_is_running() {
                utils::start_ollama_serve();
                utils::wait_for_ollama_ready(60);
            }

            // 3. Prepara o modelo
            let _ = tx.send("Configurando modelo 'rustops' (isso pode demorar se for o primeiro download)...".to_string());
            utils::setup_custom_model();

            // 4. Sinaliza que terminou
            let _ = tx.send("CONCLUIDO".to_string());
        });

        Self {
            user_input: String::new(),
            db: AppDatabase::carregar(),
            receptor_de_texto: None,
            is_processing: false,
            editando_titulo: false,
            novo_titulo_temp: String::new(),
            
            is_initialized: false,
            startup_receiver: Some(rx),
            startup_status_text: "Iniciando RustOps...".to_string(),
        }
    }
}

// =========================================================
// MÉTODOS PRIVADOS DE DESENHO DA INTERFACE
// =========================================================
impl RustOpsApp {
    fn desenhar_tela_carregamento(&mut self, ctx: &egui::Context) -> bool {
        // Se já carregou tudo, avisa o update() para desenhar o resto do app
        if self.is_initialized {
            return false;
        }

        // Verifica se há novas atualizações da thread de setup
        if let Some(rx) = &self.startup_receiver {
            // Usamos 'while let' para ler todas as mensagens pendentes rapidamente
            while let Ok(msg) = rx.try_recv() {
                if msg == "CONCLUIDO" {
                    self.is_initialized = true;
                    self.startup_receiver = None; // Limpa o canal da memória
                    return false;
                } else {
                    self.startup_status_text = msg;
                }
            }
        }

        // Desenha a tela de loading bonitona
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                
                ui.heading(egui::RichText::new("🚀 RustOps").size(40.0).strong());
                ui.add_space(40.0);
                
                ui.spinner(); // O círculo girando
                ui.add_space(20.0);
                
                // O texto dinâmico que vem do utils.rs
                ui.label(egui::RichText::new(&self.startup_status_text).size(16.0));
                ui.add_space(30.0);
                
                // Aviso de segurança para acalmar o usuário ansioso
                ui.label(
                    egui::RichText::new("A primeira execução pode levar vários minutos (baixando motor de IA de 4GB).\nPor favor, não feche o aplicativo.")
                        .color(egui::Color32::YELLOW)
                );
            });
        });

        // Força a tela a atualizar em 60fps enquanto carrega
        ctx.request_repaint();
        true 
    }

    fn processar_mensagens_ia(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.receptor_de_texto {
            if let Ok(pedaco_texto) = rx.try_recv() {
                if pedaco_texto == "[FIM]" {
                    self.is_processing = false;
                    self.receptor_de_texto = None;
                    self.db.salvar();
                } else {
                    let sessao_atual = self.db.get_sessao_ativa_mut();
                    if let Some(ultima_msg) = sessao_atual.mensagens.last_mut() {
                        if ultima_msg.role == "assistant" {
                            ultima_msg.content.push_str(&pedaco_texto);
                        }
                    }
                }
                ctx.request_repaint();
            }
        }
    }

    fn desenhar_painel_lateral(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("menu_lateral")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Conversas");
                ui.add_space(10.0);

                if ui.button("➕ Novo Chat").clicked() {
                    self.db.criar_nova_sessao();
                }
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut id_clicado = None;
                    for sessao in &self.db.sessoes {
                        let is_active = sessao.id == self.db.sessao_ativa_id;
                        if ui.selectable_label(is_active, &sessao.titulo).clicked() {
                            id_clicado = Some(sessao.id);
                        }
                    }
                    if let Some(id) = id_clicado {
                        self.db.sessao_ativa_id = id;
                        self.db.salvar();
                    }
                });
            });
    }

    fn desenhar_rodape(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("rodape").show(ctx, |ui| {
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                let response = ui.add_sized(
                    [ui.available_width() - 80.0, 30.0],
                    egui::TextEdit::singleline(&mut self.user_input).hint_text("Digite sua mensagem aqui..."),
                );
                let button = ui.add_sized([70.0, 30.0], egui::Button::new("Enviar"));

                if (button.clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) 
                    && !self.user_input.is_empty() && !self.is_processing 
                {
                    let historico_completo = {
                        let sessao_atual = self.db.get_sessao_ativa_mut();
                        sessao_atual.mensagens.push(ChatMessage {
                            role: "user".to_string(),
                            content: self.user_input.clone(),
                        });
                        sessao_atual.mensagens.push(ChatMessage {
                            role: "assistant".to_string(),
                            content: "".to_string(),
                        });
                        sessao_atual.mensagens.clone()
                    }; 

                    self.db.salvar();
                    self.user_input.clear();
                    self.is_processing = true;

                    let (tx, rx) = channel();
                    self.receptor_de_texto = Some(rx);

                    thread::spawn(move || {
                        ollama::send_to_ollama_chat(historico_completo, tx);
                    });
                }
            });
            ui.add_space(10.0);

            // Assinatura
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("Desenvolvido por Danilo Ferreira Sousa | Versão: 1.1 | Motor: Mistral")
                    .small()
                    .color(egui::Color32::DARK_GRAY)
                );
            });
            ui.add_space(10.0);
        });
    }

    fn desenhar_painel_central(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Cabeçalho Dinâmico
            ui.horizontal(|ui| {
                if self.editando_titulo {
                    ui.add(egui::TextEdit::singleline(&mut self.novo_titulo_temp));
                    if ui.button("✅ Salvar").clicked() {
                        self.db.renomear_sessao_ativa(self.novo_titulo_temp.clone());
                        self.editando_titulo = false;
                    }
                } else {
                    let titulo_chat = self.db.get_sessao_ativa_mut().titulo.clone();
                    ui.heading(format!("RustOps - {}", titulo_chat));
                    
                    if ui.button("✏️").clicked() {
                        self.editando_titulo = true;
                        self.novo_titulo_temp = titulo_chat;
                    }
                    if self.db.sessoes.len() > 1 {
                        if ui.button("🗑️").clicked() {
                            self.db.deletar_sessao_ativa();
                        }
                    }
                }
            });
            ui.separator();

            // Área de Mensagens
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for msg in &self.db.get_sessao_ativa_mut().mensagens {
                        if msg.role == "system" { continue; }

                        let mut texto_exibicao = if msg.role == "user" {
                            format!("Você: {}", msg.content)
                        } else {
                            format!("RustOps: {}", msg.content)
                        };

                        let cor_texto = if msg.role == "user" {
                            egui::Color32::LIGHT_BLUE
                        } else {
                            egui::Color32::LIGHT_GREEN
                        };

                        ui.add(
                            egui::TextEdit::multiline(&mut texto_exibicao)
                                .text_color(cor_texto)
                                .frame(false)
                                .desired_width(ui.available_width())
                        );
                        ui.add_space(5.0);
                    }

                    if self.is_processing {
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(egui::RichText::new("RustOps está digitando...").color(egui::Color32::GRAY));
                        });
                    }
                });
        });
    }
}

// =========================================================
// O LOOP PRINCIPAL DA INTERFACE (Módulo eframe)
// =========================================================
impl eframe::App for RustOpsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        
        // 1. TELA DE CARREGAMENTO
        // Se retornar true, significa que a tela de carregamento está ativa e
        // as outras partes do app não devem ser desenhadas ainda.
        if self.desenhar_tela_carregamento(ctx) {
            return;
        }

        // 2. PROCESSAMENTO EM SEGUNDO PLANO
        self.processar_mensagens_ia(ctx);

        // 3. DESENHO DOS PAINÉIS
        self.desenhar_painel_lateral(ctx);
        self.desenhar_rodape(ctx);
        self.desenhar_painel_central(ctx);
    }
}