pub mod gerenciador_prompts;
pub mod motor_ia;
pub mod db_sessao;
pub mod ferramentas;
pub mod nucleo;

pub use nucleo::NucleoAlfabetizacao;

uniffi::setup_scaffolding!();
