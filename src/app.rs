use eframe::egui;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

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

    pub aceitou_termos: bool,

    // Variaveis para o atualizador:
    pub receptor_update: Option<Receiver<String>>,
    pub versao_disponivel: Option<String>,

    // Contrle da janela de apoio
    pub mostrar_janela_apoio: bool,
    pub markdown_cache: CommonMarkCache,
}

// =========================================================
// INICIALIZAÇÃO E THREAD DE CARREGAMENTO
// =========================================================
impl RustOpsApp {
    pub fn new() -> Self {
        let (tx, rx) = channel::<String>();
        let (tx_update, rx_update) = channel::<String>();

        thread::spawn(move || {
            // Closure imediata para facilitar o tratamento de erros com '?'
            let setup_result = (|| -> Result<(), String> {
                let _ = tx.send("Verificando motod de IA...".to_string());
                if !utils::is_ollama_installed() {
                    let _ = tx.send("Instalando Ollama...".to_string());
                    utils::instalar_ollama()?;
                }

                let _ = tx.send("Iniciando serviço...".to_string());
                if !utils::ollama_is_running() {
                    utils::start_ollama_serve();
                    if !utils::wait_for_ollama_ready(60) {
                        return Err("O motor de IA não respondeu a tempo.".to_string());
                    }
                }

                // Nova chamada que utiliza o Result e o tx para status.
                utils::setup_custom_model(&tx)?;
                Ok(())
            })();

            if let Err(e) = setup_result {
                let _ = tx.send(format!("ERRO_FATAL: {}", e));
            } else {
                let _ = tx.send("CONCLUIDO".to_string());
            }
        });

        // Thread rodando em segundo plano (Verificador do GitHub)
        let versao_atual = env!("CARGO_PKG_VERSION").to_string();
        thread::spawn(move ||{
            let url = "https://api.github.com/repos/daniloferreirasousa/rustops-gui/releases/latest";

            let client = reqwest::blocking::Client::builder()
                .user_agent("RustOps-App")
                .build()
                .unwrap();

            if let Ok(resposta) = client.get(url).send() {
                if let Ok(json) = resposta.json::<serde_json::Value>() {
                    if let Some(tag) = json["tag_name"].as_str() {
                        let tag_limpa = tag.trim_start_matches('v');
                        if tag_limpa != versao_atual {
                            let _ = tx_update.send(tag.to_string());
                        }
                    }
                }
            }
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
            aceitou_termos: false,

            receptor_update: Some(rx_update),
            versao_disponivel: None,

            mostrar_janela_apoio: false,

            markdown_cache: CommonMarkCache::default(),
        }
    }
}

// =========================================================
// MÉTODOS PRIVADOS DE DESENHO DA INTERFACE
// =========================================================
impl RustOpsApp {
    fn termos_de_uso(&mut self, ctx: &egui::Context) -> bool {
        if !self.aceitou_termos {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 3.0);

                    ui.heading("⚠️ AVISO LEGAL E TERMOS DE USO");
                    ui.add_space(20.0);

                    ui.label("O RustOps é uma ferramenta local desenvolvida estritamente para fins educacionais");
                    ui.label("e de pesquisa em Segurança da Informação (Red Teaming).");
                    ui.add_space(10.0);

                    ui.label("O desenvolvedor não se responsabilida por nenhum dano, uso indevido");
                    ui.label("ou atividade ilegal realizada com o auxilio desta ferramenta.");
                    ui.add_space(10.0);

                    ui.label("Ao utilizar o RustOps, você concorda que todas as ações tomadas");
                    ui.label("são de sua única e exclusiva responsabilidade.");
                    ui.add_space(30.0);

                    // Botão que destrava o aplicativo
                    if ui.button("🚨 Eu li, compreendo e aceito os termos").clicked() {
                        self.aceitou_termos = true;
                    }
                });
            });
            return true;
        }
        false
    }


    fn desenhar_alerta_atualizacao(&mut self, ctx: &egui::Context) {
        // 1. Tenta ler a mensage da thread do Github
        if let Some(rx) = &self.receptor_update {
            if let Ok(nova_versao) = rx.try_recv() {
                self.versao_disponivel = Some(nova_versao);
                self.receptor_update = None; // Limpa o canal
            }
        }

        // 2. Se tem versão nova, desenha uma barra superior de destaque
        if let Some(versao) = &self.versao_disponivel {
            egui::TopBottomPanel::top("painel_atualizacao").show(ctx, |ui| {
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("🚀 Nova versão do RustOps disponível ({})!", versao))
                            .color(egui::Color32::YELLOW)
                            .strong()
                    );

                    // Empurra o botão para a direita
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                       
                       if ui.button("Baixar Atualização").clicked() {
                            let url_release = "https://github.com/daniloferreirasousa/rustops-gui/releases/latest";
                            let _ = webbrowser::open(url_release);
                       }
                    });
                });
                ui.add_space(5.0);
            });
        }
    }

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
                } else if msg.starts_with("ERRO_FATAL") {
                    self.startup_status_text = msg.replace("ERRO_FATAL: ","Erro: ");
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
                
                // Se for um erro,  não mostra o spinner
                if !self.startup_status_text.starts_with("Erro:") {
                    ui.spinner();
                    ui.add_space(20.0)
                }

                // O texto dinâmico que vem do utils.rs
                ui.label(egui::RichText::new(&self.startup_status_text).size(16.0));

                if self.startup_status_text.starts_with("Erro:") {
                    ui.add_space(30.0);
                    ui.label(egui::RichText::new("Por favor, verificque sua internet e reinicie o aplicativo.").color(egui::Color32::LIGHT_RED));
                } else {
                    ui.add_space(30.0);
                    ui.label(egui::RichText::new("A primeira execução pode levar vários minutos.\nPor favor, não feche o aplicativo.").color(egui::Color32::LIGHT_YELLOW));
                }
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


                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    if ui.button("☕ Apoie o Projeto").clicked() {
                        self.mostrar_janela_apoio = true;
                    }
                    ui.add_space(10.0);
                    ui.separator();
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
                        
                        // 1. Aiciona a mensagem do usuário
                        sessao_atual.mensagens.push(ChatMessage {
                            role: "user".to_string(),
                            content: self.user_input.clone(),
                        });

                        // 2. Clona o Histórico aqui (Só vai até o "user")
                        let historico_para_api = sessao_atual.mensagens.clone();

                        // 3. Adiciona a mensagem vazia apenas para a Interface (UI) desenhar na tela
                        sessao_atual.mensagens.push(ChatMessage {
                            role: "assistant".to_string(),
                            content: "".to_string(),
                        });

                        // Retorna o histórico limpa para enviar para o Ollama
                        historico_para_api
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
                    egui::RichText::new(format!("Desenvolvido por {}", env!("CARGO_PKG_AUTHORS")))
                    .small()
                    .color(egui::Color32::DARK_GRAY)
                );
                ui.label(egui::RichText::new(format!("RustOps v{}", env!("CARGO_PKG_VERSION")))
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
                    
                    // ID da sessão atual
                    let id_sessao = self.db.sessao_ativa_id;

                    // Iter().enumarate para ter um indice (0,1,2,...) para cada mensagem
                    for (indice, msg) in self.db.get_sessao_ativa_mut().mensagens.iter().enumerate() {
                        if msg.role == "system" { continue; }

                        if msg.role == "user" {
                            // Mensagem do usuário (texto normal azulzinho)
                            ui.label(
                                egui::RichText::new(format!("Você: {}", msg.content))
                                    .color(egui::Color32::LIGHT_BLUE)
                            );
                        } else {
                            // Mensagem da IA
                            ui.label(
                                egui::RichText::new("RustOps:")
                                    .color(egui::Color32::LIGHT_GREEN)
                            );

                            // Criar um id único juntando o ID do chat + posição da mensagem
                            let id_mensagem = format!("chat_{}_msg_{}", id_sessao, indice);

                            ui.push_id(&id_mensagem, |ui| {
                                CommonMarkViewer::new()
                                    .show(ui, &mut self.markdown_cache, &msg.content);
                            });
                        }

                        // Um espaço e uma linha para separar as mensagens de forma elegante
                        ui.add_space(5.0);
                        ui.separator();
                        ui.add_space(5.0);
                    }

                    if self.is_processing {
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(egui::RichText::new("RustOps está digitando...").color(egui::Color32::DARK_GRAY));
                        });
                    }
                });
        });
    }


    fn desenhar_janela_apoio(&mut self, ctx: &egui::Context) {
        // Se for false, nem desenha nada
        if !self.mostrar_janela_apoio {
            return;
        }

        let mut aberta = self.mostrar_janela_apoio;

        egui::Window::new("☕ Apoie o Projeto")
            .open(&mut aberta) // Adiciona um "X" para fechar a janela
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]) // Abre bem no meio da tela
            .show(ctx, |ui|{
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label("Gostou do RustOps? Ele é gratuito e open-source!");
                    ui.label("Considere me pagar um café para ajudar a manter o projeto.");
                    ui.add_space(20.0);

                    let chave_pix = "00020126580014BR.GOV.BCB.PIX013693cc5dfd-0c3a-4e80-b087-4ac00a96b62e5204000053039865802BR5925DANILO DE ANDRADE FERREIR6007RESENDE62070503***63048F81";

                    if ui.button("Copiar Chave PIX").clicked() {
                        ui.ctx().copy_text(chave_pix.to_string());
                    }
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("Danilo Ferreira Sousa").small().color(egui::Color32::GRAY));
                    ui.add_space(10.0);
                });
            });
            self.mostrar_janela_apoio = aberta;
    }
}

// =========================================================
// O LOOP PRINCIPAL DA INTERFACE (Módulo eframe)
// =========================================================
impl eframe::App for RustOpsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        

        // 1. TELAS DE BLOQUEIO (Loading e Termos)
        if self.desenhar_tela_carregamento(ctx) { return; }
        if self.termos_de_uso(ctx) { return; }

        // 1.5 ALERTAS
        self.desenhar_alerta_atualizacao(ctx);

        // 2. PROCESSAMENTO EM SEGUNDO PLANO
        self.processar_mensagens_ia(ctx);

        // 3. DESENHO DOS PAINÉIS
        self.desenhar_painel_lateral(ctx);
        self.desenhar_rodape(ctx);
        self.desenhar_painel_central(ctx);

        // 4. JANELAS FLUTUANTES (Modais)
        self.desenhar_janela_apoio(ctx);
    }
}