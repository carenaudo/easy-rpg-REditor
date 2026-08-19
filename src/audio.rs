use std::fs::File;
use std::path::Path;

/// Windows MCI ("sequencer") backend for `.mid`/`.midi` playback. Routes
/// through whichever MIDI synth Windows has configured - the built-in
/// Microsoft GS Wavetable Synth on a stock install - which is also what the
/// original RPG Maker 2000/2003 games were authored against, so this needs
/// no bundled soundfont.
#[cfg(windows)]
mod mci {
    use std::os::windows::ffi::OsStrExt;
    use std::panic;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::{self, Sender};
    use std::sync::{Arc, Mutex};
    use windows_sys::Win32::Media::Multimedia::mciSendStringW;

    const ALIAS: &str = "reditor_midi";

    fn wide(s: &str) -> Vec<u16> {
        std::ffi::OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    fn send(cmd: &str) -> Result<(), String> {
        let wcmd = wide(cmd);
        let ret = unsafe { mciSendStringW(wcmd.as_ptr(), std::ptr::null_mut(), 0, std::ptr::null_mut()) };
        if ret == 0 {
            Ok(())
        } else {
            Err(format!("MCI error {ret} for command: {cmd}"))
        }
    }

    fn open_and_play(path: &Path) -> Result<(), String> {
        stop_and_close();
        // MCI's command grammar treats `"` as the path delimiter, so a path
        // containing one would truncate the command - strip it rather than
        // trying to escape it, since Windows paths can't contain `"` anyway.
        let path_str = path.to_string_lossy().replace('"', "");
        send(&format!("open \"{path_str}\" type sequencer alias {ALIAS}"))?;
        send(&format!("play {ALIAS}"))
    }

    fn stop_and_close() {
        let _ = send(&format!("stop {ALIAS}"));
        let _ = send(&format!("close {ALIAS}"));
    }

    fn set_volume_raw(v: u32) {
        let _ = send(&format!("setaudio {ALIAS} volume to {}", v.min(1000)));
    }

    enum Command {
        OpenAndPlay(PathBuf),
        Stop,
        SetVolume(u32),
    }

    /// Every Win32 MCI call runs on this dedicated worker thread rather than
    /// whichever thread issues the command (the egui UI thread, in
    /// practice). `mciSendStringW` is synchronous with no documented upper
    /// bound on latency - calling it directly from the UI thread means any
    /// stall (a slow/uncooperative driver, a malformed file, device
    /// contention) freezes the whole app for as long as it takes, which
    /// reads as a crash. Commands are fire-and-forget from the caller's
    /// side; failures land in `last_error` for the UI to poll on a later
    /// frame instead of being returned synchronously. `catch_unwind` around
    /// each command means even a genuine panic in this FFI boundary can't
    /// take the whole process down with it - the worker just reports the
    /// failure and keeps processing the next command.
    pub struct Worker {
        tx: Sender<Command>,
        last_error: Arc<Mutex<Option<String>>>,
        active: Arc<AtomicBool>,
    }

    impl Worker {
        pub fn spawn() -> Self {
            let (tx, rx) = mpsc::channel::<Command>();
            let last_error = Arc::new(Mutex::new(None));
            let active = Arc::new(AtomicBool::new(false));
            let last_error_bg = Arc::clone(&last_error);
            let active_bg = Arc::clone(&active);

            let spawned = std::thread::Builder::new().name("reditor-midi".to_string()).spawn(move || {
                for cmd in rx {
                    let is_open = matches!(cmd, Command::OpenAndPlay(_));
                    let is_stop = matches!(cmd, Command::Stop);
                    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| match &cmd {
                        Command::OpenAndPlay(path) => open_and_play(path),
                        Command::Stop => {
                            stop_and_close();
                            Ok(())
                        }
                        Command::SetVolume(v) => {
                            set_volume_raw(*v);
                            Ok(())
                        }
                    }));

                    match result {
                        Ok(Ok(())) => {
                            if is_open {
                                active_bg.store(true, Ordering::SeqCst);
                            } else if is_stop {
                                active_bg.store(false, Ordering::SeqCst);
                            }
                            *last_error_bg.lock().unwrap() = None;
                        }
                        Ok(Err(e)) => {
                            active_bg.store(false, Ordering::SeqCst);
                            *last_error_bg.lock().unwrap() = Some(e);
                        }
                        Err(_) => {
                            active_bg.store(false, Ordering::SeqCst);
                            *last_error_bg.lock().unwrap() = Some("MIDI playback failed unexpectedly.".to_string());
                        }
                    }
                }
            });

            // A failure to spawn a thread at all is a real environment
            // problem, not something MIDI-specific - fall back to a worker
            // whose channel is immediately disconnected, so callers just see
            // their sends silently no-op instead of panicking here.
            let tx = match spawned {
                Ok(_) => tx,
                Err(_) => {
                    let (dead_tx, _dead_rx_dropped_here) = mpsc::channel::<Command>();
                    dead_tx
                }
            };

            Self { tx, last_error, active }
        }

        pub fn open_and_play(&self, path: &Path) {
            let _ = self.tx.send(Command::OpenAndPlay(path.to_path_buf()));
        }

        pub fn stop(&self) {
            let _ = self.tx.send(Command::Stop);
        }

        pub fn set_volume(&self, v: u32) {
            let _ = self.tx.send(Command::SetVolume(v));
        }

        pub fn is_active(&self) -> bool {
            self.active.load(Ordering::SeqCst)
        }

        pub fn take_last_error(&self) -> Option<String> {
            self.last_error.lock().unwrap().take()
        }
    }
}

#[cfg(not(windows))]
mod mci {
    use std::path::Path;

    pub struct Worker;

    impl Worker {
        pub fn spawn() -> Self {
            Self
        }
        pub fn open_and_play(&self, _path: &Path) {}
        pub fn stop(&self) {}
        pub fn set_volume(&self, _v: u32) {}
        pub fn is_active(&self) -> bool {
            false
        }
        pub fn take_last_error(&self) -> Option<String> {
            None
        }
    }
}

/// A single-track audio previewer used by the Sound Test dialog and the
/// Resource Manager's Music/Sound categories. Wraps rodio 0.22's
/// `MixerDeviceSink` + `Player` pair - the sink handle must stay alive for
/// as long as playback should be possible (dropping it closes the output
/// device), so both live together in this struct for the app's lifetime.
///
/// `.mid`/`.midi` files don't go through rodio at all (it has no MIDI
/// synthesizer) - they're routed to the background `mci::Worker` instead.
pub struct AudioPlayer {
    _handle: rodio::MixerDeviceSink,
    player: rodio::Player,
    midi: mci::Worker,
}

impl AudioPlayer {
    /// Opens the default audio output device. Returns `None` (never panics)
    /// if there's no usable device - missing audio hardware shouldn't be
    /// fatal to the rest of the editor, same as an unconfigured RTP path.
    pub fn new() -> Option<Self> {
        let handle = rodio::DeviceSinkBuilder::open_default_sink().ok()?;
        let player = rodio::Player::connect_new(&handle.mixer());
        Some(Self { _handle: handle, player, midi: mci::Worker::spawn() })
    }

    /// Stops whatever is playing and starts `path` from the beginning.
    /// Routes `.mid`/`.midi` through the Windows MCI worker (fire-and-forget
    /// - see `mci::Worker` for why); everything else through rodio directly.
    pub fn play_file(&self, path: &Path) -> Result<(), String> {
        if Self::is_midi(path) {
            self.player.stop();
            self.midi.open_and_play(path);
            return Ok(());
        }

        self.midi.stop();
        self.player.stop();
        let file = File::open(path).map_err(|e| format!("Couldn't open {}: {e}", path.display()))?;
        let source = rodio::Decoder::try_from(file).map_err(|e| format!("Couldn't decode {}: {e}", path.display()))?;
        self.player.append(source);
        self.player.play();
        Ok(())
    }

    pub fn stop(&self) {
        self.midi.stop();
        self.player.stop();
    }

    /// `v` is 0.0-1.0, matching the Sound Test dialog's 0-100% slider.
    /// Applies to MIDI too (remapped to MCI's 0-1000 scale) while a MIDI
    /// track is the one actually active.
    pub fn set_volume(&self, v: f32) {
        if self.midi.is_active() {
            self.midi.set_volume((v.clamp(0.0, 1.0) * 1000.0) as u32);
        } else {
            self.player.set_volume(v);
        }
    }

    /// `v` is a speed multiplier (1.0 = normal), matching the Pitch slider.
    /// This changes playback rate, not true pitch-shifting - close enough
    /// for a preview tool, and it's what rodio's `Player` actually offers.
    /// No-op while MIDI is active: MCI's sequencer device doesn't expose a
    /// tempo control through this command set, so callers should also
    /// disable the Pitch slider whenever a MIDI track is loaded.
    pub fn set_speed(&self, v: f32) {
        if self.midi.is_active() {
            return;
        }
        self.player.set_speed(v);
    }

    /// Surfaces the most recent MIDI playback failure, if any, clearing it
    /// once returned. The worker thread can't return errors synchronously
    /// (that's the point of backgrounding it - see `mci::Worker`), so
    /// callers should poll this once per frame while a MIDI track might be
    /// loading or playing, and show it the same way a synchronous
    /// `play_file` error would be shown.
    pub fn take_midi_error(&self) -> Option<String> {
        self.midi.take_last_error()
    }

    /// True for `.mid`/`.midi` files, which rodio cannot decode (no
    /// synthesizer) - these are routed through the platform MCI backend by
    /// `play_file` instead. Callers should also use this to disable the
    /// Pitch slider for MIDI, since `set_speed` is a no-op for it.
    pub fn is_midi(path: &Path) -> bool {
        matches!(
            path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref(),
            Some("mid") | Some("midi")
        )
    }
}
