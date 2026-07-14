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
    let mut streamed_saudacao = String::new();
    let saudacao = nucleo.iniciar_interacao_stream(&id_crianca, |chunk| {
        streamed_saudacao.push_str(chunk);
        print!("{}", chunk);
        io::stdout().flush().unwrap();
    });
    
    let sobra_saudacao = saudacao.strip_prefix(&streamed_saudacao).unwrap_or(&saudacao);
    if !sobra_saudacao.is_empty() {
        print!("{}", sobra_saudacao);
        io::stdout().flush().unwrap();
    }
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
        
        let mut streamed_resposta = String::new();
        let resposta = nucleo.processar_entrada_stream(&id_crianca, entrada, |chunk| {
            streamed_resposta.push_str(chunk);
            print!("{}", chunk);
            io::stdout().flush().unwrap();
        });
        
        let sobra_resposta = resposta.strip_prefix(&streamed_resposta).unwrap_or(&resposta);
        if !sobra_resposta.is_empty() {
            print!("{}", sobra_resposta);
            io::stdout().flush().unwrap();
        }
        println!("\n");
    }
}
