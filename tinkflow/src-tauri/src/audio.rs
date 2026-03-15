use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
use rubato::{FftFixedIn, Resampler};
use std::sync::{Arc, Mutex};
use std::thread;
use crossbeam_channel::{unbounded, Receiver, Sender};

pub trait VoiceActivityDetector: Send + Sync {
    fn is_active(&mut self, audio_chunk: &[f32]) -> bool;
}

pub struct RmsVad {
    threshold: f32,
}

impl RmsVad {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }
}

impl VoiceActivityDetector for RmsVad {
    fn is_active(&mut self, audio_chunk: &[f32]) -> bool {
        if audio_chunk.is_empty() {
            return false;
        }
        let sum_squares: f32 = audio_chunk.iter().map(|&x| x * x).sum();
        let rms = (sum_squares / audio_chunk.len() as f32).sqrt();
        rms > self.threshold
    }
}

pub struct AudioCapturer {
    buffer: Arc<Mutex<Vec<f32>>>,
    stream: Option<cpal::Stream>,
    stop_tx: Option<Sender<()>>,
    vad: Arc<Mutex<Box<dyn VoiceActivityDetector>>>,
    settings_manager: Arc<crate::settings::SettingsManager>,
}

pub fn list_input_devices() -> Vec<String> {
    let host = cpal::default_host();
    let mut names = vec!["default".to_string()];
    if let Ok(devices) = host.input_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                if !names.contains(&name) {
                    names.push(name);
                }
            }
        }
    }
    names
}

impl AudioCapturer {
    pub fn new(vad_threshold: f32, settings_manager: Arc<crate::settings::SettingsManager>) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
            stream: None,
            stop_tx: None,
            vad: Arc::new(Mutex::new(Box::new(RmsVad::new(vad_threshold)))),
            settings_manager,
        }
    }

    pub fn start_recording(&mut self) -> Result<(), String> {
        if self.stream.is_some() {
            return Ok(()); // Already recording
        }

        let host = cpal::default_host();
        let device_name = self.settings_manager.get().audio_device_name;
        println!("[Audio] Requested device: '{}'", device_name);
        
        // Find device by name — error loudly if not found
        let device = if device_name == "default" || device_name.is_empty() {
            let dev = host.default_input_device()
                .ok_or("No default input device found on this system")?;
            if let Ok(name) = dev.name() {
                println!("[Audio] Using default device: '{}'", name);
            }
            dev
        } else {
            let mut found_device = None;
            if let Ok(devices) = host.input_devices() {
                for d in devices {
                    if let Ok(name) = d.name() {
                        if name == device_name {
                            found_device = Some(d);
                            break;
                        }
                    }
                }
            }
            match found_device {
                Some(dev) => {
                    println!("[Audio] Matched device: '{}'", device_name);
                    dev
                }
                None => {
                    // List what IS available to help debug
                    let available = list_input_devices().join(", ");
                    return Err(format!(
                        "Microphone '{}' not found. Available devices: [{}]. \
                         Check Settings or reconnect your device.",
                        device_name, available
                    ));
                }
            }
        };

        let config = device.default_input_config().map_err(|e| e.to_string())?;

        let channels = config.channels();
        let sample_rate = config.sample_rate();
        let sample_format = config.sample_format();

        println!("Started recording: {} Hz, {} channels", sample_rate, channels);

        self.buffer.lock().unwrap().clear();
        let shared_buffer = self.buffer.clone();
        
        // Channel to send raw audio chunks from CPAL to the processing thread
        let (audio_tx, audio_rx): (Sender<Vec<f32>>, Receiver<Vec<f32>>) = unbounded();
        // Channel to signal the processing thread to stop
        let (stop_tx, stop_rx): (Sender<()>, Receiver<()>) = unbounded();
        
        self.stop_tx = Some(stop_tx);

        // CPAL data callback matching multiple formats to f32
        let err_fn = |err| eprintln!("An error occurred on stream: {}", err);
        
        let stream = match sample_format {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _: &_| {
                    let mono_data: Vec<f32> = data.chunks(channels as usize)
                        .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
                        .collect();
                    let _ = audio_tx.send(mono_data);
                },
                err_fn,
                None,
            ).map_err(|e| e.to_string())?,
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _: &_| {
                    let mono_data: Vec<f32> = data.chunks(channels as usize)
                        .map(|chunk| chunk.iter().map(|&s| s.to_sample::<f32>()).sum::<f32>() / channels as f32)
                        .collect();
                    let _ = audio_tx.send(mono_data);
                },
                err_fn,
                None,
            ).map_err(|e| e.to_string())?,
            _ => return Err(format!("Unsupported sample format: {:?}", sample_format)),
        };

        stream.play().map_err(|e| e.to_string())?;
        self.stream = Some(stream);

        // Clone the VAD so we can use it in the thread
        let vad_clone = self.vad.clone();

        // Spawn a background thread to handle resampling and buffering
        thread::spawn(move || {
            let target_sample_rate = 16000;
            let mut resampler = if sample_rate != target_sample_rate {
                // FftFixedIn is fast and good for audio
                // rubato chunk size must match the requested input size.
                // We will buffer the input into chunks of 1024 to feed the resampler.
                Some(FftFixedIn::<f32>::new(
                    sample_rate as usize,
                    target_sample_rate as usize,
                    1024,
                    1,
                    1,
                ).unwrap())
            } else {
                None
            };

            let mut input_buffer: Vec<f32> = Vec::new();

            loop {
                // Check if we should stop
                if stop_rx.try_recv().is_ok() {
                    break;
                }

                // Try to get audio from the callback
                if let Ok(mut chunk) = audio_rx.try_recv() {
                    if let Some(res) = resampler.as_mut() {
                        input_buffer.append(&mut chunk);
                        
                        let required_len = res.input_frames_next();
                        while input_buffer.len() >= required_len {
                            let process_chunk: Vec<f32> = input_buffer.drain(0..required_len).collect();
                            let mut output = res.process(&[process_chunk], None).unwrap();
                            // Apply VAD on the 16kHz resampled chunk
                            let output_chunk = &mut output[0];
                            
                            let mut is_active = false;
                            if let Ok(mut vad_guard) = vad_clone.lock() {
                                is_active = vad_guard.is_active(output_chunk);
                            }
                            
                            if is_active {
                                shared_buffer.lock().unwrap().append(output_chunk);
                            }
                        }
                    } else {
                        // Already 16kHz, apply VAD and append
                        let mut is_active = false;
                        if let Ok(mut vad_guard) = vad_clone.lock() {
                            is_active = vad_guard.is_active(&chunk);
                        }
                        
                        if is_active {
                            shared_buffer.lock().unwrap().append(&mut chunk);
                        }
                    }
                } else {
                    // Small sleep to prevent burning CPU in the loop
                    thread::sleep(std::time::Duration::from_millis(5));
                }
            }
        });

        Ok(())
    }

    pub fn stop_recording(&mut self) -> Result<Vec<f32>, String> {
        if let Some(stream) = self.stream.take() {
            stream.pause().map_err(|e| e.to_string())?;
            println!("Stopped recording. Stream dropped.");
        }
        
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }

        // Return a copy of the recorded 16kHz float buffer
        let final_audio = self.buffer.lock().unwrap().clone();
        println!("Captured {} audio samples", final_audio.len());
        Ok(final_audio)
    }
}
