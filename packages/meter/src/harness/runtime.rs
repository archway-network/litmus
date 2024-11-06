use crate::console::Console;
use crate::harness::graph_builder::{Graph, Storage};
use crate::job::Job;
use crate::naming::NameType;
use crate::results::FinalizedGroup;
use litmus_chain::ArchwayApp;
use tokio::runtime::Builder;

pub struct HarnessRuntime {
    pub(crate) results: Vec<FinalizedGroup>,
    pub(crate) storage: Box<dyn Storage>,
    pub(crate) graphs: Vec<Box<dyn Graph>>,
}

impl HarnessRuntime {
    pub fn new(
        groups: Vec<(String, NameType)>,
        jobs: Vec<Box<dyn Job>>,
        mut console: Box<dyn Console>,
        storage: Box<dyn Storage>,
        graphs: Vec<Box<dyn Graph>>,
        mut tokio_builder: Builder,
    ) -> Self {
        // TODO: unsafe unwrap
        tokio_builder.build().unwrap().block_on(async {
            let mut running_jobs = vec![];

            // Add all jobs into the runtime
            let mut console_setup = vec![];
            for (group, _) in groups.iter() {
                console_setup.push((group.clone(), 0));
            }

            for job in jobs {
                let g = console_setup.get_mut(job.get_group_id()).unwrap();
                g.1 += 1;

                running_jobs.push(tokio::spawn(async move {
                    let app = ArchwayApp::new();
                    job.run(app)
                }));
            }

            console.init("TODO: add title?".to_string(), console_setup);

            // Iterate through all the jobs waiting for them to complete
            let mut results = vec![vec![]; groups.len()];
            let mut index = 0;
            while !running_jobs.is_empty() {
                if running_jobs[index].is_finished() {
                    let job = running_jobs.remove(index);
                    // TODO: unsafe unwrap
                    let mut result = job.await.unwrap();
                    console.increment(result.group_id);
                    results[result.group_id].append(&mut result.results);
                }

                index += 1;
                if index >= running_jobs.len() {
                    index = 0;
                }
            }

            let mut group_results = vec![];
            for ((group, name_type), results) in groups.iter().zip(results) {
                group_results.push(FinalizedGroup {
                    group: group.to_string(),
                    name_type: *name_type,
                    results,
                })
            }

            Self {
                results: group_results,
                storage,
                graphs,
            }
        })
    }

    pub fn save(&self) {
        self.storage.save(&self.graphs, &self.results)
    }
}
