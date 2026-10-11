# multi-agent-jev

Template genérico e reutilizável para orquestração multi-agente com **JEV (TypeSafe System One)** como camada de decisão rápida (L0).

## Arquitetura

```
tarefa → jev-core (route/risk/escalate/brief, tipado)
       → orchestrator (fluxo L1→L4 + git worktrees)
       → agentes (skills + contratos tipados)
```

Camadas: L0 Decisão | L1 Design (Maestro) | L2 Implementação (Codesmith) | L3 Revisão adversarial (Codex) | L4 Merge (Humano único gate).

## Estrutura

```
crates/jev-core/      # L0: cliente Jev + roteamento + risco + escalação
crates/agents/        # contratos tipados (DesignDoc, ReviewReport, ...)
crates/orchestrator/  # worktrees, fila, transições de estado
skills/               # prompt seeds versionados
docs/SPEC.md          # especificação
AGENTS.md             # contrato canônico
templates/project/    # esqueleto para novos projetos
```

## Desenvolvimento

```bash
source $HOME/.cargo/env
cargo test --workspace --tests
cargo build --workspace
```

## Adotar em novo projeto

1. Copie `templates/project/` para o novo repositório (ou use este repo como template)
2. Ajuste `AGENTS.md` aos princípios do projeto (sem violar não-negociáveis)
3. Defina limiares e caminhos críticos em `jev-core`
4. Habilite branch protection: exigir aprovação humana em `main`

## Princípios

- Humano é o único merge gate
- Maestro escreve design antes de código
- Fluxo exclusivo via git worktrees
- AGENTS.md é contrato canônico
- Jev produz apenas schemas tipados (sem texto livre)
- IA assistiva, nunca decisória em ações irreversíveis
