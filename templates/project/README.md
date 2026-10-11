# Template do projeto

Este diretório contém o esqueleto mínimo para adotar `multi-agent-jev` em um novo projeto.

## Setup rápido

1. Copiar este diretório para a raiz do novo repositório
2. Ajustar `AGENTS.md` conforme necessidades (respeitando princípios não-negociáveis)
3. Criar worktrees/CI conforme docs/SPEC.md
4. Habilitar branch protection: aprovação humana obrigatória em `main`

## Estrutura sugerida

- `docs/` — documentação
- `docs/designs/` — designs L1 (obrigatório antes de código)
- `skills/` — seeds específicos do projeto (podem herdar do template)

Ver `AGENTS.md` para contrato canônico.
