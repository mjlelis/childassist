use projeto_alfabetizacao_rust::nucleo::NucleoAlfabetizacao;
use std::io::{self, Write};

fn main() {
    println!("=======================================================");
    println!("🚀 CHILDASSIST - NÚCLEO DE ALFABETIZAÇÃO (IoT Edition)");
    println!("=======================================================");
    println!("Iniciando sistema embarcado...");

    // Caminhos relativos assumindo execução na raiz do repositório
    let caminho_config = "dados/config.json".to_string();
    let dir_prompts = "dados/prompts".to_string();
    let caminho_db = "dados/sessao_iot.sqlite".to_string();
    let caminho_dicionario = "dados/dicionario_ptbr.json".to_string();
    let caminho_proibidas = "dados/palavras_proibidas.txt".to_string();

    let nucleo = match NucleoAlfabetizacao::new(
        caminho_config,
        dir_prompts,
        caminho_db,
        caminho_dicionario,
        caminho_proibidas,
    ) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Erro crítico ao iniciar o núcleo: {}", e);
            std::process::exit(1);
        }
    };

    let id_crianca = "crianca_iot_1".to_string();

    print!("🤖 Brinquedo: ");
    io::stdout().flush().unwrap();
    let _saudacao = nucleo.iniciar_interacao_stream(&id_crianca, |chunk| {
        print!("{}", chunk);
        io::stdout().flush().unwrap();
    });
    println!("\n");

    loop {
        print!("Criança: ");
        io::stdout().flush().unwrap();

        let mut entrada = String::new();
        io::stdin().read_line(&mut entrada).unwrap();
        
        let entrada = entrada.trim();
        if entrada.eq_ignore_ascii_case("sair") {
            println!("Encerrando o sistema...");
            break;
        }

        if entrada.is_empty() {
            continue;
        }

        print!("🤖 Brinquedo: ");
        io::stdout().flush().unwrap();
        let _resposta = nucleo.processar_entrada_stream(&id_crianca, entrada, |chunk| {
            print!("{}", chunk);
            io::stdout().flush().unwrap();
        });
        println!("\n");
    }
}
