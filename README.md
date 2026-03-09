# RustOps GUI - Ferramenta Educacional de Red Team com IA Local

**RustOps GUI** é uma interface gráfica nativa, rápida e independente desenvolvida em Rust para interagir com modelos de linguagem locais via Ollama. O foco do projeto é fornecer uma experiência "plug and play", gerenciando as suas próprias dependências de infraestrutura de forma invisível para o usuário final, operando 100% offline e com alta performance. Esta ferramenta foi desenhada estritamente para fins educacionais e de pesquisa em Segurança da Informação.

### 🛡️ Por que escolher o RustOps?

* **Privacidade Absoluta (100% Offline):** Seus dados, seus prompts e seus relatórios nunca saem da sua máquina. Não há telemetria, não há assinaturas mensais e nenhum dado é enviado para a nuvem.
* **Ambiente Livre de Filtros Corporativos:** IAs comerciais frequentemente bloqueiam prompts legítimos de cibersegurança e análise de código. O RustOps utiliza um modelo local focado em engenharia, garantindo que suas pesquisas teóricas de *Red Teaming* não sejam interrompidas.
* **Zero Configuração:** Esqueça tutoriais complexos envolvendo Python ou Docker. O aplicativo é independente e gerencia sua própria infraestrutura em background com apenas um clique.

### 🚀 O que há de novo na v0.1.4?

* **Motor de IA Otimizado:** Implementação do modelo `nchapman/dolphin3.0-qwen2.5:3b`. Esta mudança garante um equilíbrio superior entre velocidade de resposta, menor consumo de RAM e alta capacidade de raciocínio lógico em Rust e cibersegurança.
* **Fluxo de Inicialização Robusto:** Tratamento de erros aprimorado na thread de setup, com logs detalhados e notificação clara de falhas na tela de carregamento.
* **Arquitetura "Zero-Touch" Refinada:** Otimização na criação do `Modelfile` customizado, garantindo que o modelo `rustops` seja configurado de forma consistente em qualquer máquina.



### ✨ Funcionalidades Atuais

* **Segurança e Proteção Legal:** Termos de Uso com aceite obrigatório para isenção de responsabilidade.
* **Verificador de Atualizações Inteligente:** Consulta automática à API do GitHub em *background* com notificação visual.
* **Automação de Infraestrutura (Zero-Touch):** Detecção, instalação e gerenciamento do serviço Ollama via `pkexec` ou `PowerShell`.
* **Interface Gráfica Nativa Assíncrona:** Construída com `eframe`/`egui` para renderização leve.
* **Gerenciamento de Conversas:** Painel lateral para criar, alternar e renomear sessões com persistência local em arquivo.
* **Streaming e Markdown:** Efeito "máquina de escrever" e suporte completo a renderização de blocos de código.

### 🗂️ Arquitetura do Projeto

O projeto segue os rígidos princípios de *Separation of Concerns*:

```text
rustops_gui/
├── Cargo.toml          # Gerenciador de dependências e metadados
└── src/
    ├── main.rs         # Entry point: Gerencia o ciclo de vida da janela.
    ├── app.rs          # Interface (egui): Splash screen, estados e lógica de UI.
    ├── storage.rs      # Banco de Dados: Persistência e gestão de sessões.
    ├── ollama.rs       # Rede: Cliente HTTP, streaming e JSON.
    └── utils.rs        # Infraestrutura: Comandos de SO, detecção e setup.
```
### 🚀 Como Executar e Distribuir

#### Para Desenvolver (Modo Debug):

```Bash
cargo run
```
#### Para Distribuir no Linux (Gerar Instalador .deb):
```Bash
cargo deb
```
Para Distribuir no Windows (Cross-Compile):
```Bash
cargo build --target x86_64-pc-windows-gnu --release
```
### 🗺️ Roadmap (Evolução do Projeto)
- [x] Streaming de texto em tempo real (Canais MPSC).

- [x] Memória de contexto (Histórico de sessão).

- [x] Persistência de Dados (Salvar conversas localmente).

- [x] Múltiplos Chats: Gerenciamento no painel lateral.

- [x] Automação Multiplataforma (Zero-Touch Setup).

- [x] Verificador de Atualizações via GitHub API.

- [x] Renderização de Markdown com syntax highlighting.

- [x] Migração para Dolphin 3.0 (3B) para otimização de performance.

### 📄 Licença
Este projeto está licenciado sob a GNU General Public License v3.0 (GPLv3). Software livre e de código aberto. Consulte o arquivo LICENSE para detalhes.

## ☕ Apoie o Projeto
Se curtiu o projeto, ele é gratuito e open-source! Considere me pagar um café apontando a câmera do seu celular para o QR Code abaixo ou usando a chave PIX:

<img src="assets/pix.png" alt="QR Code PIX" width="200">

* **Chave PIX (Copia e Cola)**: 
00020126580014BR.GOV.BCB.PIX013693cc5dfd-0c3a-4e80-b087-4ac00a96b62e5204000053039865802BR5925DANILO DE ANDRADE FERREIR6007RESENDE62070503***63048F81