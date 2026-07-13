# Guia de Contribuição

Obrigado pelo seu interesse em contribuir para o nosso Núcleo de IA em Rust! Este guia visa ajudar a manter a qualidade e os padrões do projeto orgânicos e organizados.

## 1. Como Contribuir

Trabalhamos com o fluxo de Pull Requests (PRs). Para contribuir:
1. Faça o fork do repositório.
2. Crie uma branch com a sua feature: `git checkout -b feature/minha-feature`.
3. Faça os commits seguindo nossos padrões de mensagem.
4. Envie a branch para o seu fork: `git push origin feature/minha-feature`.
5. Abra um Pull Request contra a nossa branch `main`.

## 2. Nomes de Branches

Utilize os seguintes prefixos para nomear suas branches, seguido de um nome descritivo (em minúsculas, separado por hífens):
- `feature/` - Para novas funcionalidades.
- `bugfix/` - Para correções de bugs em produção.
- `chore/` - Para tarefas de manutenção ou atualizações de dependências.
- `docs/` - Para atualizações em documentação.

Exemplo: `feature/nova-sanitizacao-palavroes`

## 3. Mensagens de Commit (Conventional Commits)

Nós adotamos a convenção de **Conventional Commits**. Sua mensagem de commit deve seguir o formato:

```
<tipo>: <descrição curta>

[Opcional] <descrição detalhada do que foi feito e por que>
```

**Tipos válidos:**
- `feat`: Uma nova funcionalidade
- `fix`: Correção de um bug
- `docs`: Apenas mudanças na documentação
- `style`: Alterações de formatação (espaços em branco, formatação, etc) que não afetam a lógica
- `refactor`: Uma mudança no código que não corrige bugs nem adiciona features (ex: refatoração)
- `test`: Adição de testes faltantes ou correção de testes existentes
- `chore`: Atualização de ferramentas de build, dependências ou rotinas secundárias

**Exemplos:**
- `feat: adiciona controle dinâmico de temperatura no roteamento`
- `fix: corrige acesso concorrente no db de sessões`
- `docs: atualiza guia de contribuição com novas tags`

## 4. Padrões de Código em Rust

Sempre execute os seguintes comandos antes de abrir o seu PR. O nosso CI (GitHub Actions) irá barrar PRs que não passarem nessas validações:

- **Formatação:** 
  Verifique se o seu código obedece o estilo oficial:
  ```bash
  cargo fmt
  ```

- **Linter (Clippy):** 
  Garanta que não há "code smells" ou problemas de performance:
  ```bash
  cargo clippy -- -D warnings
  ```

- **Testes:** 
  Tenha certeza de que tudo compila e que os testes locais (se houver) estão passando:
  ```bash
  cargo test
  ```

Agradecemos imensamente por dedicar seu tempo e conhecimento para melhorar a educação infantil através da tecnologia de ponta!
