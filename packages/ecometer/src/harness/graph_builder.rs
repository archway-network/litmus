use crate::naming::NameType;
use crate::results::{BenchResults, FinalizedGroup};
use plotters::chart::LabelAreaPosition;
use plotters::element::Rectangle;
use plotters::prelude::{
    ChartBuilder, Color, IntoDrawingArea, LineSeries, Palette, Palette99, SVGBackend, BLACK, WHITE,
};
use std::collections::BTreeMap;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, BufWriter};
use std::ops::Range;
use std::path::PathBuf;

#[macro_export]
macro_rules! pkg_version {
    () => {
        format!(
            "v{}_{}_{}",
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR"),
            env!("CARGO_PKG_VERSION_PATCH")
        )
    };
}

#[derive(Clone)]
pub enum HistoryLimit {
    OrderedLimit { limit: usize },
    RotateName { rotation: Vec<String> },
    None,
}

pub struct GenericStorage {
    // Where the files are stored
    pub path: String,
    // Limit for history
    pub limit: HistoryLimit,
    // New results name
    pub new_results: String,
}

impl GenericStorage {
    pub fn save_last(path: &str) -> Self {
        Self {
            path: path.to_string(),
            limit: HistoryLimit::RotateName {
                rotation: vec!["new".to_string(), "base".to_string()],
            },
            new_results: "new".to_string(),
        }
    }

    pub fn package_version(path: &str, limit: usize, version: String) -> Self {
        Self {
            path: path.to_string(),
            limit: HistoryLimit::OrderedLimit { limit },
            new_results: version,
        }
    }
}

impl Storage for GenericStorage {
    fn save(&self, graphs: &[Box<dyn Graph>], results: &[FinalizedGroup]) {
        let mut path = PathBuf::from(&self.path);
        create_dir_all(&path).unwrap();

        path.push("config.json");
        let config_path = path.clone();
        path.pop();

        // Load storage
        let mut config = if let Some(file) = File::open(&config_path).ok() {
            let cfg: BTreeMap<String, Vec<FinalizedGroup>> =
                serde_json::from_reader(BufReader::new(&file)).unwrap();
            cfg
        } else {
            BTreeMap::new()
        };

        match &self.limit {
            HistoryLimit::OrderedLimit { limit } => {
                while config.len() > *limit {
                    config.pop_first();
                }
            }
            HistoryLimit::RotateName { rotation } => {
                let mut last = None;
                for rotate in rotation.iter() {
                    let target = last.unwrap_or(&self.new_results);

                    if let Some(removed) = config.remove(target) {
                        config.insert(rotate.to_string(), removed);
                    }

                    last = Some(rotate);
                }
            }
            HistoryLimit::None => {}
        }

        // TODO: create graphs

        // Go through the current results' groups
        for group in results.iter() {
            let mut data = vec![(self.new_results.clone(), group.results.clone())];
            // Search for the same groups in the stored config
            for (name, results) in config.iter() {
                if let Some(matching_group) = results
                    .iter()
                    .find(|matching_group| matching_group.group == group.group)
                {
                    // Filter out if name types changed
                    if matching_group.name_type == group.name_type {
                        data.push((name.clone(), matching_group.results.clone()));
                    }
                }
            }

            for graph in graphs.iter() {
                graph.graph(path.to_str().unwrap(), &group.group, group.name_type, &data)
            }
        }

        // Save config
        config.insert(self.new_results.clone(), results.to_vec());
        serde_json::to_writer_pretty(BufWriter::new(File::create(&config_path).unwrap()), &config)
            .unwrap();
    }
}

pub trait Storage {
    fn save(&self, graphs: &[Box<dyn Graph>], results: &[FinalizedGroup]);
}

pub trait Graph {
    /// Makes a graph of title group, and organized all the data with name type. each bundle of results must have unique names
    fn graph(
        &self,
        path: &str,
        group: &str,
        name_type: NameType,
        results: &[(String, BenchResults)],
    );
}

// TODO: modify so users can pick between arch, gas wanted and used
// TODO: maybe it should take a pathbuf and implement its own filenames

#[derive(Copy, Clone)]
pub enum GraphTarget {
    GasWanted,
    GasUsed,
    ArchSpent,
}

pub struct LinearGraph {
    /// Image size
    pub size: (u32, u32),
    pub left_label_size: u32,
    pub bottom_label_size: u32,
    pub caption_size: u32,
    pub y_padding: u64,
    pub target: GraphTarget,
}

impl Default for LinearGraph {
    fn default() -> Self {
        Self {
            size: (1024, 1024),
            left_label_size: 80,
            bottom_label_size: 40,
            caption_size: 40,
            y_padding: 1000,
            target: GraphTarget::ArchSpent,
        }
    }
}

impl Graph for LinearGraph {
    fn graph(
        &self,
        path: &str,
        group: &str,
        name_type: NameType,
        results: &[(String, BenchResults)],
    ) {
        let mut save = PathBuf::from(path);
        save.push(format!("{}.svg", group));

        let area = SVGBackend::new(&save, self.size).into_drawing_area();
        area.fill(&WHITE).unwrap();

        let y_spec = LinearGraph::y_spec(self.target, self.y_padding, results);
        let x_spec = match name_type {
            NameType::Numbered => {
                let labels: Vec<usize> = results[0]
                    .1
                    .iter()
                    .map(|n| n.name.parse::<usize>().unwrap())
                    .collect();

                0..labels.iter().max().unwrap() + 1
            }
            NameType::Named => 0..results[0].1.len() + 1,
        };

        let mut ctx = ChartBuilder::on(&area)
            .set_label_area_size(LabelAreaPosition::Left, self.left_label_size)
            .set_label_area_size(LabelAreaPosition::Bottom, self.bottom_label_size)
            .caption(&group, ("sans-serif", self.caption_size))
            .build_cartesian_2d(x_spec, y_spec)
            .unwrap();

        let mut mesh = ctx.configure_mesh();

        mesh.x_desc("Group_Bench").y_desc(match self.target {
            GraphTarget::GasWanted => "Gas_Wanted",
            GraphTarget::GasUsed => "Gas_Used",
            GraphTarget::ArchSpent => "aarch",
        });

        if name_type == NameType::Named {
            mesh.x_label_formatter(&|x| {
                results
                    .get(*x)
                    .map(|(s, _)| s.to_string())
                    .unwrap_or("".to_string())
            })
            .draw()
            .unwrap();
        } else {
            mesh.draw().unwrap();
        }

        // Only used for named
        let labels = match name_type {
            NameType::Numbered => vec![],
            NameType::Named => results[0].1.iter().map(|a| a.name.clone()).collect(),
        };

        for (idx, (key, value)) in results.iter().enumerate() {
            let color = Palette99::pick(idx).mix(0.9);

            // Temp fix
            // let items = value.iter().map(|(name, data)| (name, data.gas_used));
            let mut items = vec![];
            for data in value.iter() {
                let val = match self.target {
                    GraphTarget::GasWanted => data.gas.wanted,
                    GraphTarget::GasUsed => data.gas.used,
                    GraphTarget::ArchSpent => data.arch,
                };

                let x = match name_type {
                    NameType::Numbered => data.name.parse::<usize>().unwrap(),
                    NameType::Named => {
                        if let Some(idx) = labels.iter().position(|l| l == &data.name) {
                            idx
                        } else {
                            continue;
                        }
                    }
                };

                items.push((x, val));
            }

            // Sort iterative items to prevent malformed lines
            // match name_type {
            //     NameType::Numbered => {}
            //     NameType::Named => items.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap())
            // }
            items.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());

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
    }
}

impl LinearGraph {
    fn y_spec(target: GraphTarget, padding: u64, data: &[(String, BenchResults)]) -> Range<u128> {
        let mut all = vec![];
        for (_, d) in data {
            for result in d {
                all.push(match target {
                    GraphTarget::GasWanted => result.gas.wanted,
                    GraphTarget::GasUsed => result.gas.used,
                    GraphTarget::ArchSpent => result.arch,
                });
            }
        }
        all.sort();

        let mut max_gas = *all.last().unwrap();
        let mut min_gas = *all.first().unwrap();

        let size_difference = (max_gas - min_gas) / 4;
        // Avoid bugs
        if size_difference <= 1 || min_gas == max_gas {
            min_gas = min_gas.checked_sub(padding as u128).unwrap_or(0);
            max_gas = max_gas.checked_add(padding as u128).unwrap_or(u128::MAX);
        }

        (min_gas.checked_sub(size_difference).unwrap_or(0))
            ..max_gas.checked_add(size_difference).unwrap_or(u128::MAX)
    }
}
