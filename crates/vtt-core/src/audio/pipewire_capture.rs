//! PipeWire native audio capture
use super::{AudioResult, AudioFormat};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::mem;

// User data passed to PipeWire callbacks
struct CaptureData {
    buffer: Arc<Mutex<Vec<f32>>>,
    format: pipewire::spa::param::audio::AudioInfoRaw,
    active: Arc<Mutex<bool>>,
}

/// PipeWire audio capture using native API
/// 
/// Spawns a thread to run the PipeWire main loop when capturing starts.
pub struct PipeWireCapture {
    format: AudioFormat,
    buffer: Arc<Mutex<Vec<f32>>>,
    active: Arc<Mutex<bool>>,
    thread_handle: Option<JoinHandle<()>>,
}

impl PipeWireCapture {
    pub fn new() -> AudioResult<Self> { Self::with_format(AudioFormat::DEFAULT) }
    
    pub fn with_format(format: AudioFormat) -> AudioResult<Self> {
        Ok(Self { 
            format,
            buffer: Arc::new(Mutex::new(Vec::new())),
            active: Arc::new(Mutex::new(false)),
            thread_handle: None,
        })
    }
    
    pub fn start(&mut self) -> AudioResult<()> {
        let mut active = self.active.lock().unwrap();
        if *active {
            return Ok(());
        }
        *active = true;
        drop(active);
        
        // Clone Arcs for the new thread
        let buffer_clone = self.buffer.clone();
        let active_clone = self.active.clone();
        
        // Spawn PipeWire event loop thread
        let handle = thread::spawn(move || {
            Self::run_pipewire_loop(buffer_clone, active_clone);
        });
        
        self.thread_handle = Some(handle);
        
        // Give PipeWire a moment to initialize
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        Ok(())
    }
    
    pub fn stop(&mut self) -> AudioResult<()> {
        let mut active = self.active.lock().unwrap();
        if !*active {
            return Ok(());
        }
        *active = false;
        drop(active);
        
        // Wait for the thread to finish
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
        
        Ok(())
    }
    
    pub fn take_buffer(&mut self) -> Vec<f32> {
        let mut b = self.buffer.lock().unwrap();
        std::mem::take(&mut *b)
    }
    
    pub fn buffer_len(&self) -> usize { 
        self.buffer.lock().unwrap().len() 
    }
    
    pub fn is_active(&self) -> bool { 
        *self.active.lock().unwrap() 
    }
    
    pub fn format(&self) -> &AudioFormat { &self.format }
    
    // Runs the PipeWire event loop in a separate thread
    fn run_pipewire_loop(
        buffer: Arc<Mutex<Vec<f32>>>,
        active: Arc<Mutex<bool>>,
    ) {
        use pipewire as pw;
        use pw::spa;
        use pw::spa::pod::Pod;
        
        // Initialize PipeWire
        pw::init();
        
        // Create main loop
        let mainloop = match pw::main_loop::MainLoopRc::new(None) {
            Ok(ml) => ml,
            Err(e) => {
                eprintln!("PipeWire: Failed to create main loop: {}", e);
                return;
            }
        };
        
        // Create context
        let context = match pw::context::ContextRc::new(&mainloop, None) {
            Ok(ctx) => ctx,
            Err(e) => {
                eprintln!("PipeWire: Failed to create context: {}", e);
                return;
            }
        };
        
        // Connect to PipeWire daemon
        let core = match context.connect_rc(None) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("PipeWire: Failed to connect: {}", e);
                return;
            }
        };
        
        // Create user data
        let data = CaptureData {
            buffer,
            format: Default::default(),
            active: active.clone(),
        };
        
        // Create stream properties
        let props = pw::properties::properties! {
            *pw::keys::MEDIA_TYPE => "Audio",
            *pw::keys::MEDIA_CATEGORY => "Capture",
            *pw::keys::MEDIA_ROLE => "Music",
        };
        
        // Create stream
        let stream = match pw::stream::StreamBox::new(&core, "vtt-capture", props) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("PipeWire: Failed to create stream: {}", e);
                return;
            }
        };
        
        // Set up callbacks
        let _listener = stream
            .add_local_listener_with_user_data(data)
            .param_changed(|_, user_data, id, param| {
                // NULL means to clear the format
                let Some(param) = param else {
                    return;
                };
                
                if id != pw::spa::param::ParamType::Format.as_raw() {
                    return;
                }
                
                let (media_type, media_subtype) = match pw::spa::param::format_utils::parse_format(param) {
                    Ok(v) => v,
                    Err(_) => return,
                };
                
                // Only accept raw audio
                if media_type != pw::spa::param::format::MediaType::Audio
                    || media_subtype != pw::spa::param::format::MediaSubtype::Raw
                {
                    return;
                }
                
                // Parse the format
                if let Err(e) = user_data.format.parse(param) {
                    eprintln!("PipeWire: Failed to parse format: {}", e);
                    return;
                }
                
                eprintln!(
                    "PipeWire: Capturing rate:{} channels:{}",
                    user_data.format.rate(),
                    user_data.format.channels()
                );
            })
            .process(|stream, user_data| {
                // Check if we should still be active
                if !*user_data.active.lock().unwrap() {
                    return;
                }
                
                match stream.dequeue_buffer() {
                    None => {}
                    Some(mut buffer) => {
                        let datas = buffer.datas_mut();
                        if datas.is_empty() {
                            return;
                        }
                        
                        let data = &mut datas[0];
                        let n_channels = user_data.format.channels();
                        
                        // Get chunk size before borrowing data
                        let chunk_size = data.chunk().size();
                        let n_samples = chunk_size / mem::size_of::<f32>() as u32;
                        
                        if let Some(samples) = data.data() {
                            // Convert to f32 samples and store in buffer
                            // Use first channel only (mono)
                            let mut audio_samples = Vec::with_capacity((n_samples / n_channels) as usize);
                            for n in (0..n_samples).step_by(n_channels as usize) {
                                let start = n as usize * mem::size_of::<f32>();
                                let end = start + mem::size_of::<f32>();
                                if end <= samples.len() {
                                    let chan = &samples[start..end];
                                    // Use try_into() to convert &[u8] to [u8; 4]
                                    if let Ok(bytes) = chan.try_into() {
                                        let f = f32::from_le_bytes(bytes);
                                        audio_samples.push(f);
                                    }
                                }
                            }
                            
                            // Push to shared buffer
                            let mut buf = user_data.buffer.lock().unwrap();
                            buf.extend(audio_samples);
                        }
                    }
                }
            })
            .register();
        
        // Create format parameters
        let mut audio_info = spa::param::audio::AudioInfoRaw::new();
        audio_info.set_format(spa::param::audio::AudioFormat::F32LE);
        
        let obj = pw::spa::pod::Object {
            type_: pw::spa::utils::SpaTypes::ObjectParamFormat.as_raw(),
            id: pw::spa::param::ParamType::EnumFormat.as_raw(),
            properties: audio_info.into(),
        };
        
        let values: Vec<u8> = pw::spa::pod::serialize::PodSerializer::serialize(
            std::io::Cursor::new(Vec::new()),
            &pw::spa::pod::Value::Object(obj),
        )
        .unwrap()
        .0
        .into_inner();
        
        let mut params = [Pod::from_bytes(&values).unwrap()];
        
        // Connect the stream
        if let Err(e) = stream.connect(
            spa::utils::Direction::Input,
            None,
            pw::stream::StreamFlags::AUTOCONNECT
                | pw::stream::StreamFlags::MAP_BUFFERS
                | pw::stream::StreamFlags::RT_PROCESS,
            &mut params,
        ) {
            eprintln!("PipeWire: Failed to connect stream: {}", e);
            return;
        }
        
        // Run the main loop (blocking)
        mainloop.run();
    }
}

impl Drop for PipeWireCapture {
    fn drop(&mut self) { 
        let _ = self.stop(); 
    }
}
