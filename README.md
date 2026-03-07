# RustOps GUI - Ferramenta Educacional de Red Team com IA Local

**RustOps GUI** é uma interface gráfica nativa, rápida e independente desenvolvida em Rust para interagir com modelos de linguagem locais via Ollama. O foco do projeto é fornecer uma experiência "plug and play", gerenciando as suas próprias dependências de infraestrutura de forma invisível para o usuário final, operando 100% offline e com alta performance. Esta ferramenta foi desenhada estritamente para fins educacionais e de pesquisa em Segurança da Informação.

### 🛡️ Por que escolher o RustOps?

* **Privacidade Absoluta (100% Offline):** Seus dados, seus prompts e seus relatórios nunca saem da sua máquina. Não há telemetria, não há assinaturas mensais e nenhum dado é enviado para a nuvem. Todo o processamento e histórico ficam restritos ao seu disco rígido local.
* **Ambiente Livre de Filtros Corporativos:** IAs comerciais frequentemente bloqueiam prompts legítimos de cibersegurança e análise de código. O RustOps utiliza um modelo local focado em engenharia, garantindo que suas pesquisas teóricas de *Red Teaming* não sejam interrompidas por restrições de APIs de terceiros.
* **Zero Configuração:** Esqueça tutoriais complexos envolvendo Python, dependências ou Docker. O aplicativo é independente e gerencia sua própria infraestrutura em background com apenas um clique.

### ✨ Funcionalidades Atuais

* **Segurança e Proteção Legal:**
    * **Termos de Uso (Disclaimer):** Tela de bloqueio inicial que exige o aceite explícito do usuário, garantindo a isenção de responsabilidade do desenvolvedor e reforçando o uso ético e educacional da ferramenta.
* **Verificador de Atualizações Inteligente:**
    * Consulta automática e silenciosa à API do GitHub em *background* (Thread separada).
    * Notificação visual não-obstrutiva quando uma nova *Release* está disponível, com link direto para download.
* **Automação de Infraestrutura Multiplataforma (Zero-Touch Setup):**
    * Detecta automaticamente se o Ollama está instalado.
    * **Compilação Condicional:** Lógica de instalação nativa adaptada via `#[cfg]`. Utiliza `pkexec` no Linux e `PowerShell` (UAC) no Windows para elevação de privilégios de forma segura.
    * Inicia o serviço em background automaticamente (`systemctl` ou `spawn`, dependendo do SO).
* **Gestão Dinâmica de Modelos:**
    * Gera e compila um modelo customizado (`rustops`) a partir de um Modelfile embutido no próprio código.
* **Interface Gráfica Nativa Assíncrona:**
    * Construída com `eframe`/`egui` para renderização incrivelmente rápida e leve.
    * **Tela de Carregamento Dinâmica (Splash Screen):** O aplicativo abre instantaneamente e exibe o progresso de inicialização (downloads, configurações) através de textos dinâmicos atualizados em tempo real via *threads* de segundo plano.
    * Utiliza multithreading e Canais MPSC (Message Passing) para comunicação fluida entre o backend e a interface.
* **Gerenciamento de Conversas e Persistência:**
    * **Múltiplos Chats:** Painel lateral intuitivo para criar, alternar, renomear e excluir diferentes tópicos de conversa.
    * **Persistência Local:** Todo o histórico de mensagens é salvo automaticamente no disco local, permitindo fechar e abrir o aplicativo sem perder dados.
    * **Memória de Contexto:** A IA compreende o fluxo da conversa enviando o histórico completo da sessão ativa.
* **Streaming de Respostas em Tempo Real e Markdown:**
    * Efeito "máquina de escrever": as palavras aparecem na tela assim que são geradas.
    * Suporte completo a **Markdown**, exibindo textos formatados, listas e blocos de código nativamente.
* **Distribuição Profissional Otimizada:**
    * Ocultação do terminal em ambiente de produção (aplicativo 100% gráfico).
    * Compilação com otimizações extremas (`LTO`, `strip`) garantindo executáveis minúsculos.
    * Suporte a geração de instaladores `.deb` (Linux) e executáveis standalone `.exe` (Windows) a partir do mesmo código-fonte.

### 🗂️ Arquitetura do Projeto

O projeto segue os rígidos princípios de *Separation of Concerns* (Separação de Responsabilidades), com o estado da interface refatorado em submódulos de desenho e lógica de sistema isolada:

```text
rustops_gui/
├── Cargo.toml          # Gerenciador de dependências, versão e configuração de build
└── src/
    ├── main.rs         # Ponto de entrada enxuto: Carrega o título da janela e inicia a GUI.
    ├── app.rs          # Interface (egui): Splash screen, Termos de Uso, Atualizador, e loop de eventos.
    ├── storage.rs      # Banco de Dados: Gerencia sessões, histórico e persistência no disco.
    ├── ollama.rs       # Rede: Cliente HTTP, streaming assíncrono e parsing de JSON.
    └── utils.rs        # Infraestrutura: Comandos de SO, detecção de ambiente e setup.
```

## 🚀 Como Executar e Distribuir
Pré-requisitos de Desenvolvimento:
Ter o Rust e o Cargo instalados.

Conexão com a internet na primeira execução (para baixar o Ollama e os modelos) e para o verificador de atualizações.

### Para Desenvolver (Modo Debug):
```Bash
cargo run
```
Para Distribuir no Linux (Gerar Instalador .deb):
Requer a instalação prévia da ferramenta: cargo install cargo-deb

```Bash
cargo deb
```
O arquivo de instalação estará disponível em **target/debian/**. Basta dar dois cliques para instalar no Ubuntu/Mint/Debian.

### Para Distribuir no Windows (Executável Final via Cross-Compile):

```Bash
cargo build --target x86_64-pc-windows-gnu --release
```

O executável otimizado estará em **target/x86_64-pc-windows-gnu/release/rustops-gui.exe**. O terminal de fundo é ocultado automaticamente graças à flag **windows_subsystem**.

### 🗺️ Roadmap (Evolução do Projeto)
- [x] Streaming de texto em tempo real via Canais MPSC.

- [x] Memória de contexto (Histórico de sessão).

- [x] Persistência de Dados: Salvar conversas no disco local para recuperar históricos.

- [x] Múltiplos Chats: Painel lateral para criar, gerenciar, renomear e excluir conversas.

- [x] Indicadores Visuais Dinâmicos: Tela de carregamento assíncrona para downloads pesados.

- [x] Suporte Multiplataforma: Automação de infraestrutura transparente para Linux e Windows.

- [x] Termos de Uso (Disclaimer): Bloqueio de interface para aceite de responsabilidade legal.

- [x] Atualizador Automático: Verificação de novas versões via GitHub API sem travar a interface gráfica.

- [x] Renderização de Markdown: Implementar um parser rico na interface para exibir formatações (negrito, itálico) e blocos de código com syntax highlighting nativo.

- [ ] Integração de Ícone no .exe (Windows): Utilizar build.rs e winres para embutir o ícone no binário do Windows Explorer.

## 📄 Licença

Este projeto está licenciado sob a **GNU General Public License v3.0 (GPLv3)**.

Isso significa que o RustOps é e sempre será um software livre e de código aberto. Você é livre para usar, estudar, modificar e distribuir o software. No entanto, se você distribuir uma versão modificada, **é estritamente obrigatório** que o código-fonte modificado também seja disponibilizado publicamente sob a mesma licença GPLv3. 

Para mais detalhes, consulte o arquivo [LICENSE](LICENSE) na raiz do projeto.

## ☕ Apoie o Projeto
Se curtiu o projeto, ele é gratuito e open-source! Considere me pagar um café apontando a câmera do seu celular para o QR Code abaixo ou usando a chave PIX:

<img src="assets/pix.png" alt="QR Code PIX" width="200">

* **Chave PIX (Copia e Cola)**: 00020126580014BR.GOV.BCB.PIX013693cc5dfd-0c3a-4e80-b087-4ac00a96b62e5204000053039865802BR5925DANILO DE ANDRADE FERREIR6007RESENDE62070503***63048F81