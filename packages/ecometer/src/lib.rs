pub mod console;
pub mod harness;
pub mod job;
pub mod naming;
pub mod results;

#[cfg(test)]
mod tests {
    use crate::harness::HarnessBuilder;
    use crate::job::Setup;
    use std::sync::Arc;

    #[test]
    fn generate_jobs() {
        let mut harness = HarnessBuilder::new();
        harness.continuous_group(
            "continuous",
            |_| (),
            |app, _, _| Setup {
                contract: "".to_string(),
                signer: Arc::new(app.init_account(&[]).unwrap()),
                funds: vec![],
                msg: (),
            },
            vec![0, 1, 2, 3, 4, 5],
        );

        assert_eq!(harness.jobs.len(), 1);
        assert_eq!(harness.groups.len(), 1);

        harness.independent_group(
            "independent",
            |app, _| Setup {
                contract: "".to_string(),
                signer: Arc::new(app.init_account(&[]).unwrap()),
                funds: vec![],
                msg: (),
            },
            vec![0, 1, 2, 3, 4, 5],
        );

        assert_eq!(harness.jobs.len(), 7);
        assert_eq!(harness.groups.len(), 2);
    }
}
