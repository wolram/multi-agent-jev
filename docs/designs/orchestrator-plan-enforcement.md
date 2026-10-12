# Design — Orchestrator plan enforcement

**Status:** proposto (aguarda aprovação L1/humana)
**Feature:** `orchestrator-plan-enforcement`
**Camada:** L2 (implementação em worktree `jev-orch-enforce`)

## Problema

`Orchestrator::run_once` recebe o `TaskPlan` do `jev-core` (L0) e **o descarta**
(`let _plan = ...`). Em consequência:

1. Ele avança toda tarefa por `Open→Routed→Designed→Implemented→Reviewed→ReadyToMerge`
   sem consultar `escalate.action`, `risk.critical_path` ou `needs_review`.
2. Uma decisão L0 de `escalate = human | frontier_llm` é ignorada: a tarefa roda
   localmente mesmo quando o L0 mandou escalar.
3. `needs_review` (caminho crítico ou confiança baixa) não força revisão L3 — o
   estado `Reviewed` é alcançado sem nenhum `ReviewReport`.
4. Os contratos tipados de agente (`agents::contracts::{DesignDoc, ReviewReport,
   ImplementationReport}`) existem mas não são usados pelo fluxo.

Isso viola os não-negociáveis do `AGENTS.md`: "Maestro escreve design antes de
código", "revisão adversarial obrigatória em caminho crítico" e "IA assistiva,
nunca decisória em ações irreversíveis".

## Objetivo

Tornar as decisões do L0 **vinculantes**: nenhuma transição de estado pode
contrariar o `TaskPlan`, e o gate humano (`Merged`) nunca é atingido pelo
orquestrador.

## Desenho

### Ordem das transições

```
plan = router.plan(task)
Open → Routed
  ├─ escalate != Local        → Blocked  (RunOutcome::Escalated)
  └─ escalate == Local
       worktrees.create("jev-<id>")
       design = runner.design(task, plan)
         └─ !design.approved    → Blocked  (RunOutcome::DesignRejected)
       Routed → Designed
       impl = runner.implement(task, design)
       Designed → Implemented
         ├─ needs_review && !review.approve → Blocked (RunOutcome::ReviewRejected)
         └─ Implemented → Reviewed → ReadyToMerge (RunOutcome::ReadyToMerge)
```

- `needs_review` já é calculado em `jev-core::client::plan` (cobre
  `risk.critical_path` e bandas `NeedsReview`/`Escalate`).
- `Merged` **nunca** é produzido pelo orquestrador; só o humano avança.

### Interface

Novo trait `orchestrator::runner::AgentRunner` isola a execução dos agentes
(hoje não há LLM plugado):

```rust
pub trait AgentRunner {
    fn design(&self, task: &Task, plan: &TaskPlan) -> Result<DesignDoc>;
    fn implement(&self, task: &Task, design: &DesignDoc) -> Result<ImplementationReport>;
    fn review(&self, task: &Task, report: &ImplementationReport) -> Result<ReviewReport>;
}
```

`Orchestrator<R: Router, A: AgentRunner>` ganha o campo `runner`.
`run_once` passa a devolver `RunOutcome` (não mais `bool`):

```rust
pub enum RunOutcome {
    Idle,
    Escalated { id: String, action: EscalateAction },
    DesignRejected { id: String },
    ReviewRejected { id: String },
    ReadyToMerge { id: String },
}
```

## Testes

- `local + não-crítico + design aprovado + review aprovado` → `ReadyToMerge`.
- `escalate = human` → estado `Blocked`, `Escalated{Human}`, sem worktree.
- `escalate = frontier_llm` → `Blocked`, `Escalated{FrontierLlm}`.
- `design.approved = false` → `Blocked`, `DesignRejected`.
- caminho crítico (`critical_path = true`) + `review.approve = false` →
  `Blocked`, `ReviewRejected`.
- caminho crítico + `review.approve = true` → `ReadyToMerge`.
- invariante: nenhum resultado termina em `Merged`.

## Riscos / limites

- Os limiares (`thresholds) continuam responsabilidade do `jev-core`; o
  orquestrador apenas consome `needs_review`.
- Ações irreversíveis (merge/delete/deploy) ainda não são modeladas como flag no
  `Task`; a proteção atual é estrutural (nunca chega a `Merged`). Modelar
  `irreversible` no schema fica como próximo incremento se necessário.
- Não altera o schema `jev-core`; apenas passa a usá-lo.
