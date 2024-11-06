use console::Style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

// TODO: console is very limited right now and provides now API for the user to hijack
#[allow(dead_code)]
pub struct ConsoleSettings {
    /// Combines all group progress bars into a singular bar
    single_loading_bar: bool,
    /// Spinner template
    group_template: String,
    /// Spinner tick duration
    tick_duration: Duration,
    /// Style for in-progress bars
    running_style: Style,
    /// Style for finished bars
    finished_style: Style,
    /// Spinner style
    tick_chars: String,
}

impl Default for ConsoleSettings {
    fn default() -> Self {
        Self {
            single_loading_bar: false,
            group_template: "{spinner} {prefix:.bold.dim} {percent} [{elapsed_precise}]"
                .to_string(),
            tick_duration: Duration::from_millis(50),
            running_style: Style::new().yellow(),
            finished_style: Style::new().green(),
            tick_chars: "⠁⠂⠄⡀⢀⠠⠐⠈ ".to_string(),
        }
    }
}

impl ConsoleSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn single_loading_bar(&mut self) -> &mut Self {
        self.single_loading_bar = true;
        self
    }

    pub fn with_group_template(&mut self, template: String) -> &mut Self {
        self.group_template = template;
        self
    }

    pub fn with_tick_duration(&mut self, duration: Duration) -> &mut Self {
        self.tick_duration = duration;
        self
    }

    pub fn with_tick_chars(&mut self, chars: String) -> &mut Self {
        self.tick_chars = chars;
        self
    }

    pub fn build(self) -> Box<dyn Console> {
        if self.single_loading_bar {
            Box::new(SingleConsole::new(self))
        } else {
            Box::new(GroupConsole::new(self))
        }
    }
}

pub trait Console {
    /// Initialize all group bars with their names
    fn init(&mut self, title: String, groups: Vec<(String, u64)>);
    /// Tick the progress by one
    fn increment(&mut self, group_id: usize);
}

pub struct GroupConsole {
    style: ProgressStyle,
    tick_duration: Duration,
    multi_progress: MultiProgress,
    progress_bars: Vec<ProgressBar>,
}

impl GroupConsole {
    pub fn new(settings: ConsoleSettings) -> Self {
        let style = ProgressStyle::with_template(&settings.group_template)
            .unwrap()
            .tick_chars(&settings.tick_chars);

        let pb = MultiProgress::new();

        Self {
            style,
            tick_duration: settings.tick_duration,
            multi_progress: pb,
            progress_bars: vec![],
        }
    }
}

impl Console for GroupConsole {
    fn init(&mut self, _title: String, groups: Vec<(String, u64)>) {
        for (group, len) in groups {
            let pb = ProgressBar::new(len)
                .with_style(self.style.clone())
                .with_prefix(group);
            pb.enable_steady_tick(self.tick_duration);
            self.progress_bars.push(self.multi_progress.add(pb));
        }
    }

    fn increment(&mut self, group_id: usize) {
        self.progress_bars[group_id].inc(1);
    }
}

pub struct SingleConsole {
    progress_bar: ProgressBar,
}

impl SingleConsole {
    pub fn new(settings: ConsoleSettings) -> Self {
        let style = ProgressStyle::with_template(&settings.group_template)
            .unwrap()
            .tick_chars(&settings.tick_chars);

        let pb = ProgressBar::new(0).with_style(style);
        pb.enable_steady_tick(settings.tick_duration);

        Self { progress_bar: pb }
    }
}

impl Console for SingleConsole {
    fn init(&mut self, title: String, groups: Vec<(String, u64)>) {
        self.progress_bar.set_prefix(title);
        let mut total = 0;
        for (_, t) in groups {
            total += t;
        }
        self.progress_bar.inc_length(total);
    }

    fn increment(&mut self, _group_id: usize) {
        self.progress_bar.inc(1);
    }
}
