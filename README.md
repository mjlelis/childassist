# 🚀 ChildAssist - Núcleo de Alfabetização (IoT Edition)

**ChildAssist** é um núcleo de inteligência educacional escrito puramente em **Rust**, projetado para rodar nativamente em ambientes restritos (Edge/IoT) como Raspberry Pi. O sistema atua como o "cérebro" de um brinquedo físico interativo, focando em ajudar crianças no processo de alfabetização de forma lúdica, engajadora e 100% offline.

## ✨ Características Principais

* **🚀 Alta Performance (Edge/IoT):** Desenvolvido em Rust, garantindo baixíssimo consumo de memória e inicialização instantânea, perfeito para dispositivos embarcados (Edge Computing).
* **🧠 LLM Local e Offline:** O processamento de linguagem natural ocorre **localmente**, sem necessidade de internet. Suporta múltiplos motores como `Ollama` e `Llama.cpp` nativamente, garantindo total privacidade para a criança.
* **🎭 Persona Lúdica (Professor Alfa):** A IA assume o papel do "Alfa", um assistente caloroso, enérgico e paciente. O sistema utiliza prompts projetados com engenharia cuidadosa para evitar alucinações e manter a IA sempre dentro do papel educacional.
* **🗣️ Respostas em Tempo Real (Streaming):** Suporta streaming de texto para que a criança não tenha que esperar a IA terminar de pensar antes do brinquedo começar a falar/reagir.
* **💾 Memória Persistente (SQLite):** O núcleo armazena o histórico do bate-papo, palavras já utilizadas na sessão e progresso da criança usando `rusqlite`, permitindo um acompanhamento linear.
* **🛡️ Fallbacks Resilientes:** Se o LLM falhar, a inferência demorar, ou o input da criança for inesperado (como digitar "abobrinha" em uma seleção de números), a Máquina de Estados intercepta, estabiliza o fluxo e usa recursos *hardcoded* para evitar frustração na brincadeira.

## ⚙️ Arquitetura

O sistema é construído como uma **Máquina de Estados Finita**, roteando os eventos da criança com base na intenção e no contexto armazenado no banco de dados.

* `src/nucleo.rs`: O coração da aplicação. Controla o fluxo de escolha de tema, lançamento de desafios, correções e bate_papo livre.
* `src/motor_ia.rs`: Interface polimórfica que abstrai a comunicação com LLMs (Ollama ou LLama.cpp) através de requisições nativas HTTP.
* `src/db_sessao.rs`: Camada de persistência leve SQLite focada no progresso (acertos, erros, temas esgotados).
* `dados/config.json`: Ponto central de configuração. Permite o *switch* instantâneo de provedores de IA, alteração de endpoints e controle modular das temperaturas lógicas (Criatividade x Previsibilidade).
* `dados/prompts/`: Arquivos `.txt` gerenciáveis com as diretrizes e regras comportamentais para cada etapa do jogo.

## 🛠️ Como Executar

1. **Requisitos:**
   * Rust e Cargo instalados.
   * Um servidor local de IA rodando (Ex: `Ollama` na porta 11434 ou `llama-server` na porta 8080).

2. **Configuração do LLM:**
   Abra `dados/config.json` e altere a flag `"ativo": true` para o provedor de sua preferência (ollama ou llama_cpp).

3. **Iniciando a Brincadeira:**
   No diretório do projeto, rode:
   ```bash
   cargo run
   ```

4. **Interagindo:**
   Apenas siga as instruções no terminal. Digite `1` para começar o jogo de soletrar ou `2` para bater papo.

## 💡 Fluxo de Jogo

1. **Escolha de Tema:** A criança seleciona entre 3 temas lúdicos criados dinamicamente.
2. **Missão de Soletrar:** A IA sorteia uma palavra baseada no tema, respeitando a complexidade de alfabetização e validando para nunca repetir a mesma palavra duas vezes.
3. **Avaliação:** O núcleo avalia ortograficamente. Se acertar, gera uma celebração única. Se errar, providencia um reforço positivo.
4. **Bate-Papo Livre:** A qualquer momento, a criança pode fugir do jogo e conversar sobre qualquer coisa. O LLM foi enjaulado para **não** tentar trazer o jogo de volta e deixar a criança livre no Bate Papo, provendo conversas naturais.

---
Feito com 🦀 Rust para moldar o futuro da educação infantil!
