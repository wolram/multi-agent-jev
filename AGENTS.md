# AGENTS.md — Contrato Canônico

## Propósito

Este documento é o **contrato canônico** entre agentes e humano. Nenhuma regra
importante pode viver apenas em prompts. Mudanças aqui exigem PR.

## Princípios (não-negociáveis)

1. **Humano é o único merge gate.** Branch protection em `main` exige 1 approver humano.
2. **Maestro escreve design antes de código.** Código exige `docs/designs/<feature>.md` aprovado.
3. **Fluxo exclusivo via git worktrees.** Nenhum agente trabalha em `main`.
4. **AGENTS.md é canônico.** Sincronizado via CI para projetos derivados.
5. **Jev nunca gera texto livre.** Saídas são schemas tipados. Violação = erro.
6. **IA assistiva, nunca decisória em produção.** Ações irreversíveis (merge, delete, deploy)
   nunca podem concluir como `escalate.action=local` sem validação explícita do humano.

## Camadas

- L0 (Decisão) — `jev-core`: roteia, risco, escalação. Confiança calibrada.
- L1 (Design) — Maestro
- L2 (Implementação) — Codesmith
- L3 (Revisão adversarial) — Codex (obrigatório em caminho crítico/needs_review)
- L4 (Merge) — Humano

## Workflows

### L1 → L4
1. Tarefa aberta → `jev-core.plan()` (L0)
2. L1 (Maestro): escreve design em `docs/designs/<feature>.md`
3. L2 (Codesmith): implementa em worktree `jev-<task_id>`
4. L3 (Codex): revisão adversarial (obrigatório se critical_path=true ou needs_review=true)
5. L4 (Humano): merge único gate

## Limiares de confiança

| Confiança | Ação |
|---|---|
| ≥ 0.85 | Seguir decisão Jev |
| 0.60 – 0.85 | Seguir, marcar `needs_review=true` |
| < 0.60 | Escalar (`frontier_llm` ou `human`) |

## DoD (Definition of Done)

- [ ] `cargo test --workspace --tests` verde
- [ ] Design aprovado (L1) quando aplicável
- [ ] Revisão adversarial aprovada (L3) quando exigida
- [ ] Sem trabalho direto em `main`
- [ ] Humanos aprovam PR (merge gate)

## Skills

Skills versionadas em `skills/*.md`. São seeds de prompt, não código.
Consulta: Skills → contratos tipados (`crates/agents`) → execução.

## Interpretação

Em caso de ambiguidade: **prefira princípio #1 (humano merge gate)**. Não invente
capacidades. Só execute conforme schema tipado.
