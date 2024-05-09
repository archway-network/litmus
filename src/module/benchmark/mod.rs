mod console;
mod results;
mod utils;

pub use console::{Console, ConsoleCommand};
pub use utils::*;

use crate::module::benchmark::results::{BenchResult, GroupResults};
use plotters::prelude::*;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use test_tube::{Module, Wasm};

pub struct Bench {
    pub(crate) config: BenchConfig,
    pub(crate) console: Console,
}

impl Bench {
    pub fn new() -> Self {
        Self {
            config: Default::default(),
            console: Console::init(),
        }
    }

    pub fn group(&mut self, group_name: impl Into<String>) -> Group {
        Group::new(group_name, self)
    }

    pub fn finish(self) {
        self.console.finish().unwrap();
    }
}

pub struct Group<'a> {
    benchmark: &'a mut Bench,
    name: String,
    directory: String,
    results: GroupResults,
    finished: bool,
}

fn set_ranges(min_gas: &mut u64, max_gas: &mut u64, result: &GroupResults) {
    for item in result.values() {
        *min_gas = (*min_gas).min(item.gas_used);
        *max_gas = (*max_gas).max(item.gas_used);
    }
}

impl<'a> Group<'a> {
    pub fn new(name: impl Into<String>, bench: &'a mut Bench) -> Self {
        let name = name.into();
        bench.console.init_group(name.clone());

        let directory = format!("{}/{}", bench.config.get_path(), &name);

        Self {
            benchmark: bench,
            name,
            directory,
            results: Default::default(),
            finished: false,
        }
    }

    pub fn bench<'b, I, M, P>(&'b mut self, name: impl Into<String>, init: I, params: P)
    where
        'a: 'b,
        I: Fn(&P, &Console) -> Setup<M>,
        M: Sized + Serialize,
    {
        self._bench(Some(name), init, params)
    }

    fn _bench<'b, I, M, P>(&'b mut self, name: Option<impl Into<String>>, init: I, params: P)
    where
        'a: 'b,
        I: Fn(&P, &Console) -> Setup<M>,
        M: Sized + Serialize,
    {
        let name = name.map(|s| s.into()).unwrap_or("".to_string());
        self.benchmark.console.init_bench(name.clone());

        // Run the benchmark
        let setup: Setup<M> = init(&params, &self.benchmark.console);
        let wasm = Wasm::new(&setup.app);
        let res = wasm
            .execute(&setup.contract, &setup.msg, &setup.funds, &setup.signer)
            .unwrap();
        let result = BenchResult {
            gas_wanted: res.gas_info.gas_wanted,
            gas_used: res.gas_info.gas_used,
        };

        self.results.insert(name, result);
        self.benchmark.console.finish_bench();
    }

    /// Finish the group, this will get automatically called when the Group gets dropped regardless
    pub fn finalize(&mut self) {
        self.benchmark.console.finish_group();

        fn json_file(file: &str) -> String {
            format!("{}.json", file)
        }

        // Set ranges for graphs
        let mut min_gas = u64::MAX;
        let mut max_gas = 0;

        if !self.finished {
            for config in self.benchmark.config.history.iter() {
                let mut config_path = PathBuf::from(&self.directory);
                config_path.push(&config.name);
                config_path.push("results");

                // Create missing directories
                create_dir_all(&config_path).unwrap();

                // Go through the file limit setting and remove old results
                if let Some(file_limit) = config.file_limit {
                    // Create mutable path for removal op
                    let mut res_path = config_path.clone();

                    // Load file config
                    config_path.push("config.json");
                    let file = File::open(&config_path).unwrap();

                    let mut file_config: Vec<String> =
                        serde_json::from_reader(BufReader::new(&file)).unwrap();

                    // Remove the last item until were under the file limit
                    while file_config.len() >= file_limit {
                        res_path.push(file_config.remove(0));
                        fs::remove_file(&res_path).unwrap();
                        res_path.pop();
                    }

                    // Add the new results
                    file_config.push(json_file(&config.new_results_name));
                    serde_json::to_writer_pretty(BufWriter::new(file), &file_config).unwrap();
                    config_path.pop();
                }

                let mut all_results: BTreeMap<String, GroupResults> = BTreeMap::new();

                // Go through all the stored results and load into a hashmap
                for file in fs::read_dir(&config_path).unwrap() {
                    let file_path = file.unwrap().path();
                    let file_name = file_path.iter().last().unwrap().to_str().unwrap();

                    // Ignore loading the config
                    if file_name != "config.json" {
                        let file = File::open(&file_path).unwrap();
                        let result: GroupResults =
                            serde_json::from_reader(BufReader::new(file)).unwrap();

                        all_results.insert(
                            file_name.split(".").collect::<Vec<&str>>()[0].to_string(),
                            result,
                        );
                    }
                }

                // Rotate files
                if let Some(order) = &config.file_rotation {
                    for index in 0..(order.len() - 1) {
                        let to = order.get(index).unwrap();
                        let from = order.get(index + 1).unwrap();

                        if let Some(res) = all_results.remove(from) {
                            // Rename file
                            let mut from_path = config_path.clone();
                            from_path.push(format!("{}.json", from));
                            let mut to_path = config_path.clone();
                            to_path.push(format!("{}.json", to));

                            fs::rename(&from_path, &to_path).unwrap();

                            all_results.insert(to.to_string(), res);
                        }
                    }
                }

                // Save and overwrite

                // Save new file
                config_path.push(format!("{}.json", &config.new_results_name));
                let file = File::create(&config_path).unwrap();
                serde_json::to_writer_pretty(BufWriter::new(file), &self.results).unwrap();
                config_path.pop();

                // Insert new result
                all_results.insert(config.new_results_name.clone(), self.results.clone());

                // Set ranges, we do this after file loading in the case of a removed file
                for res in all_results.values() {
                    set_ranges(&mut min_gas, &mut max_gas, res);
                }

                // Prepare graphs
                config_path.pop();
                let size_difference = (max_gas - min_gas) / 4;
                // Avoid bugs
                if size_difference <= 1 || min_gas == max_gas {
                    min_gas = min_gas.checked_sub(1000).unwrap_or(0);
                    max_gas = max_gas.checked_add(1000).unwrap_or(u64::MAX);
                }
                let y_spec = (min_gas.checked_sub(size_difference).unwrap_or(0))
                    ..max_gas.checked_add(size_difference).unwrap_or(u64::MAX);

                let labels: Vec<String> = self.results.keys().map(|k| k.clone()).collect();

                // Generate barchart for current result

                config_path.push("current_results.svg");
                let area = SVGBackend::new(&config_path, (1024, 1024)).into_drawing_area();
                area.fill(&WHITE).unwrap();

                let mut ctx = ChartBuilder::on(&area)
                    .set_label_area_size(LabelAreaPosition::Left, 80)
                    .set_label_area_size(LabelAreaPosition::Bottom, 40)
                    .caption(&self.name, ("sans-serif", 40))
                    .build_cartesian_2d(labels.as_slice().into_segmented(), y_spec.clone())
                    .unwrap();

                ctx.configure_mesh().draw().unwrap();

                ctx.draw_series(
                    Histogram::vertical(&ctx).style(BLUE.filled()).data(
                        self.results
                            .iter()
                            .map(|(name, data)| (name, data.gas_used)),
                    ),
                )
                .unwrap();

                // Drop graph items to free config_path
                drop(ctx);
                drop(area);
                config_path.pop();

                // Generate graph
                config_path.push("results.svg");
                let area = SVGBackend::new(&config_path, (1024, 1024)).into_drawing_area();
                area.fill(&WHITE).unwrap();

                let mut ctx = ChartBuilder::on(&area)
                    .set_label_area_size(LabelAreaPosition::Left, 80)
                    .set_label_area_size(LabelAreaPosition::Bottom, 40)
                    .caption(&self.name, ("sans-serif", 40))
                    .build_cartesian_2d(
                        //labels.as_slice().into_segmented(),
                        0..labels.len(), // Temp Fix
                        y_spec,
                        // (min_gas..max_gas),
                    )
                    .unwrap();

                // TODO: currently having labels with spaces breaks this
                ctx.configure_mesh()
                    .x_desc("Group_Bench")
                    .y_desc("Gas_Used")
                    .x_label_formatter(&|x| {
                        labels
                            .get(*x)
                            .map(|s| s.to_string())
                            .unwrap_or("".to_string())
                    })
                    .draw()
                    .unwrap();

                for (idx, (key, value)) in all_results.iter().enumerate() {
                    let color = Palette99::pick(idx).mix(0.9);

                    // Temp fix
                    // let items = value.iter().map(|(name, data)| (name, data.gas_used));
                    let mut items = vec![];
                    for (name, data) in value.iter() {
                        if let Some(idx) = labels.iter().position(|l| l == name) {
                            items.push((idx, data.gas_used));
                        }
                    }

                    ctx.draw_series(LineSeries::new(items, color.stroke_width(3)).point_size(2))
                        .unwrap()
                        .label(key)
                        .legend(move |(x, y)| {
                            Rectangle::new([(x, y - 5), (x + 10, y + 5)], color.filled())
                        });
                }

                ctx.configure_series_labels()
                    .border_style(BLACK)
                    .draw()
                    .unwrap();

                drop(ctx);
                drop(area);
                config_path.pop();
            }

            self.finished = true;
        }
    }
}

impl<'a> Drop for Group<'a> {
    fn drop(&mut self) {
        self.finalize()
    }
}

#[macro_export]
macro_rules! harness_main {
    ( $( $group:path ),+ $(,)* ) => {
        fn main() {
            let mut bench = $archway_test_tube::benchmark::Bench::new();

            $(
                $group(&mut bench);
            )+

            bench.finish();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::module::benchmark::console::Console;
    use crate::module::benchmark::{Bench, Setup};
    use crate::tests::netwars_msgs;
    use crate::{arch, ArchwayApp};
    use test_tube::{Account, Module, Wasm};

    fn setup(decimals: &usize, _: &Console) -> Setup<netwars_msgs::ExecuteMsg> {
        let multiplier = 10_i32.pow(*decimals as u32) as u128;

        let app = ArchwayApp::default();
        let mut accounts = app.init_accounts(&vec![arch(100 * multiplier)], 2).unwrap();
        let admin = accounts.pop().unwrap();
        let depositor = accounts.pop().unwrap();

        let wasm = Wasm::new(&app);
        let wasm_byte_code = std::fs::read("./test_artifacts/network_wars.wasm").unwrap();
        let code_id = wasm
            .store_code(&wasm_byte_code, None, &admin)
            .unwrap()
            .data
            .code_id;

        let contract_addr = wasm
            .instantiate(
                code_id,
                &netwars_msgs::InstantiateMsg {
                    archid_registry: None,
                    expiration: 604800,
                    min_deposit: arch(1 * multiplier).amount,
                    extensions: 3600,
                    stale: 604800,
                    reset_length: 604800,
                },
                Some(&admin.address()),
                Some("netwars"),
                &[],
                &admin,
            )
            .unwrap()
            .data
            .address;

        Setup::new(
            app,
            contract_addr,
            depositor,
            vec![arch(1 * multiplier)],
            netwars_msgs::ExecuteMsg::Deposit {},
        )
    }

    fn test_group(bench: &mut Bench) {
        // bench.bench("singular_bench", setup, 1);

        let mut group = bench.group("amounts_test");

        for i in 1..10 {
            group.bench(format!("{}_decimals", i), setup, i)
        }
    }

    #[test]
    fn test_main() {
        let mut bench = Bench::new();

        test_group(&mut bench);
    }
}
