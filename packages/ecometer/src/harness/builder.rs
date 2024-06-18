use crate::console::{Console, ConsoleSettings};
use crate::harness::graph_builder::{GenericStorage, Graph, LinearGraph, Storage};
use crate::harness::HarnessRuntime;
use crate::job::{Continuous, Independent, Job, Setup};
use crate::naming::{NameType, Naming};
use archway_test_tube::ArchwayApp;
use serde::Serialize;
use tokio::runtime::Builder;

pub struct HarnessBuilder {
    pub(crate) groups: Vec<(String, NameType)>,
    pub(crate) jobs: Vec<Box<dyn Job>>,
    pub(crate) console: Option<Box<dyn Console>>,
    pub(crate) storage: Option<Box<dyn Storage>>,
    pub(crate) graphs: Vec<Box<dyn Graph>>,
}

impl HarnessBuilder {
    pub fn new() -> Self {
        Self {
            groups: vec![],
            jobs: Default::default(),
            console: None,
            storage: None,
            graphs: vec![],
        }
    }

    pub fn add_group(&mut self, group: String, name_type: NameType) -> usize {
        if let Some((i, _)) = self
            .groups
            .iter()
            .enumerate()
            .find(|(_, (g, _))| g == &group)
        {
            return i;
        }

        self.groups.push((group, name_type));
        self.groups.len() - 1
    }

    pub fn continuous_group<STATE, PARAM, MSG, SETUP, UPDATE>(
        &mut self,
        name: impl ToString,
        setup: SETUP,
        update: UPDATE,
        parameters: Vec<PARAM>,
    ) where
        PARAM: Naming,
        MSG: Sized + Serialize + Send + Sync + 'static,
        PARAM: 'static + Send + Sync,
        STATE: 'static,
        SETUP: Fn(&ArchwayApp) -> STATE + 'static + Send + Sync,
        UPDATE: Fn(&ArchwayApp, &mut STATE, &PARAM) -> Setup<MSG> + 'static + Send + Sync,
    {
        let group = self.add_group(name.to_string(), PARAM::name_type());

        self.jobs.push(Box::new(Continuous {
            id: group,
            parameters,
            setup: Box::new(setup),
            update: Box::new(update),
        }));
    }

    pub fn independent_group<PARAM, MSG, SETUP>(
        &mut self,
        name: impl ToString,
        setup: SETUP,
        parameters: Vec<PARAM>,
    ) where
        PARAM: Naming + Send + Sync + 'static,
        MSG: Sized + Serialize + Send + Sync + 'static,
        SETUP: Fn(&ArchwayApp, &PARAM) -> Setup<MSG> + Copy + 'static + Send + Sync,
    {
        let setup = Box::new(setup);

        let group = self.add_group(name.to_string(), PARAM::name_type());
        for param in parameters {
            self.jobs.push(Box::new(Independent {
                id: group,
                parameters: param,
                setup: setup.clone(),
            }))
        }
    }

    pub fn set_console<T: Console + 'static>(&mut self, console: T) {
        self.console = Some(Box::new(console))
    }

    pub fn set_storage<T: Storage + 'static>(&mut self, storage: T) {
        self.storage = Some(Box::new(storage))
    }

    pub fn add_graph<T: Graph + 'static>(&mut self, graph: T) {
        self.graphs.push(Box::new(graph))
    }

    pub fn build_console(&mut self, settings: ConsoleSettings) {
        self.console = Some(settings.build())
    }

    pub fn custom_build(mut self, tokio_builder: Builder) -> HarnessRuntime {
        if self.graphs.is_empty() {
            self.graphs.push(Box::new(LinearGraph::default()));
        }

        HarnessRuntime::new(
            self.groups,
            self.jobs,
            self.console.unwrap_or(ConsoleSettings::default().build()),
            self.storage.unwrap_or(Box::new(GenericStorage::save_last("./litmus"))),
            self.graphs,
            tokio_builder,
        )
    }

    pub fn build(self) -> HarnessRuntime {
        self.custom_build(Builder::new_multi_thread())
    }
}
