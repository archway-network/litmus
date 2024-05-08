use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::mpsc::{channel, SendError, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use console::Style;

pub struct Console {
    pub conn: Sender<ConsoleCommand>,
    pub thread: JoinHandle<()>,
}

impl Console {
    pub fn init() -> Self {
        let (conn, thread) = init_console();
        Self { conn, thread }
    }

    pub fn send(&self, msg: ConsoleCommand) -> Result<(), SendError<ConsoleCommand>> {
        let res = self.conn.send(msg);
        // We need to limit how often messages can be sent to avoid duplicated in the progress
        thread::sleep(Duration::from_millis(10));
        res
    }

    pub fn finish(self) -> Result<(), SendError<ConsoleCommand>> {
        println!();
        let res = self.send(ConsoleCommand::ShutDown);
        self.thread.join().unwrap();
        res
    }
    
    pub fn init_group(&self, name: String) {
        self.send(ConsoleCommand::InitGroup { name }).unwrap()
    }

    pub fn finish_group(&self) {
        self.send(ConsoleCommand::FinishGroup).unwrap()
    }

    pub fn init_bench(&self, name: String) {
        self.send(ConsoleCommand::InitBench { name }).unwrap()
    }

    pub fn bench_msg(&self, msg: String) {
        self.send(ConsoleCommand::BenchMsg { msg }).unwrap()
    }

    pub fn finish_bench(&self) {
        self.send(ConsoleCommand::FinishBench).unwrap()
    }
}

fn init_console() -> (Sender<ConsoleCommand>, JoinHandle<()>) {
    let (sender, receiver) = channel::<ConsoleCommand>();

    let thread = thread::spawn(move || {
        let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} [{elapsed_precise}] {wide_msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
        let tick_duration = Duration::from_millis(50);
        let running_style = Style::new().yellow();
        let finished_style = Style::new().green();

        let mp = MultiProgress::new();

        let main_pb = ProgressBar::new(2)
            .with_message(format!("Bench: {}", running_style.apply_to("Running")))
            .with_style(spinner_style.clone());
        main_pb.enable_steady_tick(tick_duration);
        let main_pb = mp.add(main_pb);

        let mut group = None;
        let mut bench = None;

        loop {
            let msg = receiver.recv().unwrap();

            match msg {
                ConsoleCommand::ShutDown => {
                    main_pb.finish_with_message(format!("Bench: {}", finished_style.apply_to("Done")));
                    break;
                },
                ConsoleCommand::InitGroup { name } => {
                    let g = ProgressBar::new(2)
                        .with_message(format!("| {}: {}", &name, running_style.apply_to("Running")))
                        .with_style(spinner_style.clone());
                    g.enable_steady_tick(tick_duration);

                    group = Some((name, mp.add(g)))
                },
                ConsoleCommand::FinishGroup => {
                    if let Some((name, g)) = group {
                        g.finish_with_message(format!("| {}: {}", &name, finished_style.apply_to("Finished")));

                        group = None;
                    }
                },
                ConsoleCommand::InitBench { name } => {
                    let b = ProgressBar::new(2)
                        .with_message(format!("| | {}: {}", &name, running_style.apply_to("Running")))
                        .with_style(spinner_style.clone());
                    b.enable_steady_tick(tick_duration);

                    bench = Some((name, mp.add(b)))
                },
                ConsoleCommand::BenchMsg { msg } => {
                    if let Some((name, b)) = &bench {
                        b.set_message(format!("| | {}: {}", &name, msg));
                    }
                }
                ConsoleCommand::FinishBench => {
                    if let Some((name, b)) = bench {
                        b.finish_with_message(format!("| | {}: {}", &name, finished_style.apply_to("Finished")));

                        bench = None;
                    }
                },
            }
        }
    });

    (sender, thread)
}

pub enum ConsoleCommand {
    ShutDown,
    InitGroup { name: String },
    InitBench { name: String },
    BenchMsg { msg: String },
    FinishGroup,
    FinishBench,
}

#[cfg(test)]
mod tests {
    use crate::module::benchmark::console::{Console, ConsoleCommand};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_run() {
        let console = Console::init();
        thread::sleep(Duration::from_secs(2));

        for _ in 0..4 {
            console.send(ConsoleCommand::InitGroup { name: "some_group".to_string() }).unwrap();
            console.send(ConsoleCommand::InitBench { name: "bench_1".to_string()}).unwrap();
            console.send(ConsoleCommand::BenchMsg { msg: "Initializing".to_string() }).unwrap();
            thread::sleep(Duration::from_secs(2));
            console.send(ConsoleCommand::FinishBench).unwrap();
            console.send(ConsoleCommand::InitBench { name: "bench_2".to_string()}).unwrap();
            thread::sleep(Duration::from_secs(2));
            console.send(ConsoleCommand::FinishBench).unwrap();
            console.send(ConsoleCommand::InitBench { name: "bench_3".to_string()}).unwrap();
            thread::sleep(Duration::from_secs(2));
            console.send(ConsoleCommand::FinishBench).unwrap();
            console.send(ConsoleCommand::FinishGroup).unwrap();
        }

        console.finish().unwrap();
        thread::sleep(Duration::from_secs(2));
    }
}
