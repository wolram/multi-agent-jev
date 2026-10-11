# SPEC — multi-agent-jev

**Versão:** 0.1.0 · **Data:** 2026-10-08
**Objetivo:** template reutilizável de sistema multi-agente com fundação JEV (TypeSafe System One).

---

## 1. Visão geral

O sistema é uma **cascata arquitetural**: cada tarefa passa primeiro por uma decisão
rápida, calibrada e tipada (`jev-core`), que decide o caminho barato. Só quando a
confiança é baixa o trabalho escala para um LLM forte ou para o humano.

```
tarefa
  │
  ▼
┌─────────────────────┐
│ jev-core (Jev S1)   │  roteia, classifica risco, decide escalação
└─────────┬───────────┘
          │ briefing tipado
          ▼
┌─────────────────────┐      confiança baixa      ┌─────────────────┐
│ orchestrator L1→L4  │ ────────────────────────▶ │ LLM forte /     │
│ agentes (skills/)   │ ◀──────────────────────── │ humano (gate)   │
└─────────────────────┘      resultado            └─────────────────┘
```

## 2. Camadas

| Camada | Nome | Papel | Implementação |
|---|---|---|---|
| L0 | **Decisão** | Roteamento, risco, escalação. Nunca gera texto livre — só valores tipados. | `crates/jev-core` |
| L1 | **Design** | Escreve design antes de código (`docs/designs/<feature>.md`). | agente `maestro` |
| L2 | **Implementação** | Código em worktree isolada. | agente `codesmith` |
| L3 | **Revisão adversarial** | Segundo agente tenta quebrar o PR. Obrigatório em caminho crítico. | agente `codex` |
| L4 | **Merge humano** | Único merge gate. Nenhum agente faz merge. | humano |

## 3. Contrato Jev (L0)

### 3.1 Decisões

| ID | Decisão | Saída tipada | Uso |
|---|---|---|---|
| `route` | Qual agente trata a tarefa? | enum agente | fila do orchestrator |
| `risk` | A tarefa toca caminho crítico? | bool | exige L3 obrigatório |
| `escalate` | Resolver local / LLM forte / humano? | enum escalão | cascata |
| `brief` | Briefing estruturado para o agente | objeto | input do agente |

### 3.2 Schemas

```jsonc
// route
{ "agent": "maestro" | "codesmith" | "codex" | "researcher" | "designer",
  "confidence": 0.0..1.0 }

// risk
{ "critical_path": bool,
  "reasons": ["path:crates/jev-core/**", ...],
  "confidence": 0.0..1.0 }

// escalate
{ "action": "local" | "frontier_llm" | "human",
  "confidence": 0.0..1.0 }

// brief
{ "objective": string,
  "constraints": [string],
  "inputs": { ... },
  "done_when": [string] }
```

### 3.3 Limiares de confiança

| Confiança | Ação |
|---|---|
| `≥ 0.85` | Pode seguir com a decisão do Jev |
| `0.60 – 0.85` | Seguir, mas marcar `needs_review` na tarefa |
| `< 0.60` | Escalar para LLM forte (ou humano se o custo da decisão for alto) |

Limiares são **por ação**, não globais: uma tarefa de merge crítica tolera menos
erro que um rótulo de classificação. Constantes em `jev-core::thresholds`.

## 4. Princípios não-negociáveis

1. **Humano é o único merge gate.** Branch protection em `main` exige 1 approver humano.
2. **Maestro escreve design antes de código.** Nenhum código sem `docs/designs/<feature>.md` aprovado.
3. **Fluxo exclusivo via git worktrees.** Nenhum agente trabalha direto em `main`.
4. **AGENTS.md é canônico.** Mudanças só via PR; CI sincroniza para os projetos derivados.
5. **Jev nunca gera texto livre.** Saída é sempre schema validado; violação = erro, não retry silencioso.
6. **IA é assistiva, nunca decisória** em produção: `escalate` nunca pode concluir em `local` para ações irreversíveis (merge, delete, deploy).

## 5. Estrutura

```
multi-agent-jev/
├── AGENTS.md               contrato canônico
├── Cargo.toml              workspace
├── crates/
│   ├── jev-core/           L0: cliente Jev + roteamento + risco + escalação
│   ├── agents/             contratos tipados de input/output por agente
│   └── orchestrator/       L1→L4: fila, worktrees, ciclo de vida da tarefa
├── skills/                 prompt seeds versionados por agente
├── docs/SPEC.md            este documento
├── .github/workflows/      CI: test, sync AGENTS.md, revisão adversarial
└── templates/project/      esqueleto herdados por novos projetos
```

## 6. Ciclo de vida da tarefa

```
open → routed → designed → implemented → reviewed → ready_to_merge → merged
                     │            │            │
                     └─ blocked ◀─┴── rework ◀──┘
```

Transições válidas validadas em `orchestrator::state`. Estado ilegal = erro forte.

## 7. Definição de pronto (template)

- [ ] `cargo test` verde no workspace
- [ ] jev-core testado contra transport mockado (sem API key)
- [ ] Orchestrator com ciclo completo em teste de integração (git real, worktrees)
- [ ] AGENTS.md + skills de todos os agentes
- [ ] CI com sync de AGENTS.md e gate de revisão adversarial
- [ ] Pilot em projeto pequeno (Fase 5)
