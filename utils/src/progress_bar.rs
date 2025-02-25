use std::io::Write;
use std::sync::atomic::AtomicUsize;
#[cfg(not(feature = "wasm"))]
use std::time::Instant;

const BAR_LENGTH: usize = 50;

pub struct ProgressBar {
    title: String,
    last_progress: AtomicUsize,
    counter: AtomicUsize,
    total_count: usize,
    #[cfg(not(feature = "wasm"))]
    start: Instant,
}

impl ProgressBar {
    pub fn new(title: String, total_count: usize) -> Self {
        let ret = Self {
            title,
            total_count,
            last_progress: AtomicUsize::new(0),
            counter: AtomicUsize::new(0),
            #[cfg(not(feature = "wasm"))]
            start: Instant::now(),
        };
        ret.show_progress(0);
        ret
    }
    pub fn tic(&self) {
        let c = self
            .counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let progress = if c >= self.total_count {
            eprintln!(
                "Progress bar '{}' overflowed... ticked beyond count of {}",
                self.title, self.total_count
            );
            100
        } else {
            (100. * (c as f32) / self.total_count as f32).round() as usize
        };

        let lp = self
            .last_progress
            .load(std::sync::atomic::Ordering::Relaxed);

        let delta = if progress > lp { progress - lp } else { 0 };
        if delta >= 100 / BAR_LENGTH {
            self.last_progress
                .fetch_add(delta, std::sync::atomic::Ordering::Relaxed);
            self.show_progress(progress);
        }
    }

    fn show_progress(&self, progress: usize) {
        let filled_length = (BAR_LENGTH as f64 * (progress as f64 / 100.0)).round() as usize;
        let filled = "=".repeat(filled_length);
        let n_empty = if BAR_LENGTH >= filled_length {
            BAR_LENGTH - filled_length
        } else {
            0
        };
        let empty = " ".repeat(n_empty);
        print!(
            "\r    {} [{}{}] {:.2}%",
            self.title, filled, empty, progress
        );
        std::io::stdout().flush().unwrap();
    }

    pub fn done(&self) {
        #[cfg(not(feature = "wasm"))]
        println!("\nDone after {} seconds", self.start.elapsed().as_secs());
    }
}
