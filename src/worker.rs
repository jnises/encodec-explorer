use candle_core::Tensor;
use log::warn;

use crate::compute;

enum ToWorker {
    Stop,
    SetCodes { code: u32 },
}

enum ToApp {
    Error(String),
    Samples(Vec<f32>),
}

pub struct Worker {
    handle: Option<std::thread::JoinHandle<()>>,
    to_worker: std::sync::mpsc::Sender<ToWorker>,
    from_worker: std::sync::mpsc::Receiver<ToApp>,
    // TODO: put this somewhere else
    samples: Vec<f32>,
}

impl Worker {
    pub fn new() -> Self {
        let (to_worker, mut from_app) = std::sync::mpsc::channel();
        let (mut to_app, from_worker) = std::sync::mpsc::channel();

        let handle = Some(std::thread::spawn(move || {
            if let Err(e) = run(&mut from_app, &mut to_app) {
                to_app.send(ToApp::Error(e.to_string())).unwrap();
            }
        }));

        Self {
            handle,
            to_worker,
            from_worker,
            samples: Vec::from([0f32; 320]),
        }
    }

    pub fn set_code(&mut self, code: u32) -> anyhow::Result<()> {
        self.to_worker.send(ToWorker::SetCodes { code })?;
        Ok(())
    }

    /// returns samples if they were updated
    pub fn update(&mut self) -> Option<&[f32]> {
        let mut samples_updated = false;
        for m in self.from_worker.try_iter() {
            match m {
                ToApp::Error(e) => warn!("worker error: {e:?}"),
                ToApp::Samples(samples) => {
                    self.samples = samples;
                    samples_updated = true;
                }
            }
        }
        samples_updated.then_some(&self.samples)
    }

    pub fn samples(&self) -> &[f32] {
        self.samples.as_slice()
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.to_worker.send(ToWorker::Stop).unwrap();
        self.handle.take().unwrap().join().unwrap();
    }
}

fn run(
    from_app: &mut std::sync::mpsc::Receiver<ToWorker>,
    to_app: &mut std::sync::mpsc::Sender<ToApp>,
) -> anyhow::Result<()> {
    let c = compute::Compute::new()?;
    let mut prev_code = None;
    let mut current_code = None;
    loop {
        let r = (|| -> anyhow::Result<bool> {
            // TODO: pump all pending messages to avoid decoding things that will be immediately overwritten
            match from_app.recv()? {
                ToWorker::Stop => return Ok(false),
                ToWorker::SetCodes { code } => {
                    current_code = Some(code);
                }
            }
            if prev_code != current_code {
                if let Some(code) = current_code {
                    let pcm = c.decode_codes(code)?;
                    to_app.send(ToApp::Samples(pcm))?;
                }
                prev_code = current_code;
            }
            Ok(true)
        })();
        match r.unwrap() {
            true => {},
            false => break,
        }
        // match r {
        //     Ok(false) => break,
        //     Err(e) => {
        //         let es = e.to_string();
        //         to_app.send(ToApp::Error(es)).unwrap()
        //     },
        //     _ => {},
        // }
    }
    Ok(())
}
