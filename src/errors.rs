use thiserror::Error;

#[derive(Error, Debug)]
pub enum RustOpsError {
    
    #[error("Ollama não está rodando.")]
    OllamaNotRunning,

    #[error("Falha ao executar comando do sistema: {0}")]
    CommandExecution(String),

    #[error("Espaço em disco insuficiente. Necessário: {required}GB.")]
    InsufficientDiskSpace { required: u64 },

    #[error("Erro de rede ao conectar com o Ollama")]
    NetworkError(#[from] reqwest::Error),

    #[error("Erro de I/O: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Falha na criação do modelo: {0}")]
    ModelCreationError(String),

    #[error("falha persistente na conexão. Tente rodar 'ollama pull {0}' no terminal.")]
    ModelDownloadError(String),

    #[error("Erro inisperado: {0}")]
    Generic(String),
}