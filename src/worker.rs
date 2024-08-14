use candle_core::Tensor;

use crate::compute;

pub enum ToWorker {
    Stop,
    SetCodes{code: u32},
}

pub enum ToApp {
    Error(String),
    Samples(Vec<f32>),
}

pub struct Worker {
    handle: Option<std::thread::JoinHandle<()>>,
    to_worker: std::sync::mpsc::Sender<ToWorker>,
    from_worker: std::sync::mpsc::Receiver<ToApp>,
}

impl Worker {
    pub fn new() -> Self {
        let (mut to_worker, mut from_app) = std::sync::mpsc::channel();
        let (mut to_app, mut from_worker) = std::sync::mpsc::channel();

        let handle = Some(std::thread::spawn(move || {
            if let Err(e) = run(&mut from_app, &mut to_app) {
                to_app.send(ToApp::Error(e.to_string())).unwrap();
            }
        }));

        Self {
            handle,
            to_worker,
            from_worker,
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.to_worker.send(ToWorker::Stop).unwrap();
        self.handle.take().unwrap().join().unwrap();
    }
}

fn run(from_app: &mut std::sync::mpsc::Receiver<ToWorker>, to_app: &mut std::sync::mpsc::Sender<ToApp>) -> anyhow::Result<()> {
    let c = compute::Compute::new()?;
    let mut prev_code = None;
    let mut current_code = None;
    loop {
        match from_app.recv()? {
            ToWorker::Stop => break,
            ToWorker::SetCodes { code } => {
                current_code = Some(code);
            },
        }
        if prev_code != current_code {
            if let Some(code) = current_code {
                let code_tensor = Tensor::new(&[code], c.device())?;
                let pcm = c.decode_codes(&code_tensor)?;
                let pcm_vec: Vec<f32> = pcm.to_vec1()?;
                to_app.send(ToApp::Samples(pcm_vec))?;
            }
            prev_code = current_code;
        }
    }
    Ok(())
}