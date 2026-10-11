use crate::error::Result;

pub trait AgentRunner {
    fn design(
        &self,
        task: &jev_core::schema::Task,
        plan: &jev_core::schema::TaskPlan,
    ) -> Result<agents::contracts::DesignDoc>;

    fn implement(
        &self,
        task: &jev_core::schema::Task,
        design: &agents::contracts::DesignDoc,
    ) -> Result<agents::contracts::ImplementationReport>;

    fn review(
        &self,
        task: &jev_core::schema::Task,
        report: &agents::contracts::ImplementationReport,
    ) -> Result<agents::contracts::ReviewReport>;
}
