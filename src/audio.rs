use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use rodio::Source;
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};

pub const ERR_SOUNDFONT_MISSING: &str = "NO_SOUNDFONT";

/// A `rodio::Source` adapter that streams audio from `rustysynth`'s MIDI file sequencer.
/// Real-time sample synthesis ensures identical playback across Windows, Linux, and macOS.
pub struct MidiSource {
    sequencer: MidiFileSequencer,
    buffer: Vec<f32>,
    read_pos: usize,
    sample_rate: u32,
}

impl MidiSource {
    pub fn new(midi_path: &Path, soundfont: Arc<SoundFont>) -> Result<Self, String> {
        let midi_file_io = File::open(midi_path)
            .map_err(|e| format!("Couldn't open {}: {e}", midi_path.display()))?;
        let mut reader = BufReader::new(midi_file_io);
        let midi_file = Arc::new(
            MidiFile::new(&mut reader)
                .map_err(|e| format!("Couldn't parse MIDI file: {e:?}"))?,
        );

        let sample_rate = 44100;
        let settings = SynthesizerSettings::new(sample_rate as i32);
        let synth = Synthesizer::new(&soundfont, &settings)
            .map_err(|e| format!("Synthesizer initialization error: {e:?}"))?;

        let mut sequencer = MidiFileSequencer::new(synth);
        sequencer.play(&midi_file, false);

        Ok(Self {
            sequencer,
            buffer: Vec::with_capacity(1024),
            read_pos: 0,
            sample_rate,
        })
    }
}

impl Iterator for MidiSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.read_pos >= self.buffer.len() {
            if self.sequencer.end_of_sequence() {
                return None;
            }
            self.buffer.clear();
            let mut left = [0.0f32; 512];
            let mut right = [0.0f32; 512];
            self.sequencer.render(&mut left, &mut right);
            for i in 0..512 {
                self.buffer.push(left[i]);
                self.buffer.push(right[i]);
            }
            self.read_pos = 0;
        }

        if self.read_pos < self.buffer.len() {
            let sample = self.buffer[self.read_pos];
            self.read_pos += 1;
            Some(sample)
        } else {
            None
        }
    }
}

impl Source for MidiSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> std::num::NonZero<u16> {
        std::num::NonZero::new(2).unwrap()
    }

    fn sample_rate(&self) -> std::num::NonZero<u32> {
        std::num::NonZero::new(self.sample_rate).unwrap()
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

#[derive(Default)]
struct SoundFontState {
    soundfont: Option<Arc<SoundFont>>,
    path: Option<PathBuf>,
    error: Option<String>,
}

/// Thread-safe manager for loading, caching, and querying `.sf2` soundfonts.
#[derive(Clone, Default)]
pub struct SoundFontManager {
    inner: Arc<RwLock<SoundFontState>>,
}

impl SoundFontManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads a SoundFont from disk into memory.
    pub fn load(&self, path: &Path) -> Result<Arc<SoundFont>, String> {
        let file = File::open(path).map_err(|e| format!("Couldn't open {}: {e}", path.display()))?;
        let mut reader = BufReader::new(file);
        let sf = Arc::new(SoundFont::new(&mut reader).map_err(|e| format!("Invalid SoundFont file {}: {e:?}", path.display()))?);

        let mut state = self.inner.write().unwrap();
        state.soundfont = Some(Arc::clone(&sf));
        state.path = Some(path.to_path_buf());
        state.error = None;
        Ok(sf)
    }

    /// Unloads any currently active SoundFont.
    pub fn unload(&self) {
        let mut state = self.inner.write().unwrap();
        state.soundfont = None;
        state.path = None;
        state.error = None;
    }

    pub fn get_soundfont(&self) -> Option<Arc<SoundFont>> {
        self.inner.read().unwrap().soundfont.clone()
    }

    pub fn get_path(&self) -> Option<PathBuf> {
        self.inner.read().unwrap().path.clone()
    }

    pub fn is_loaded(&self) -> bool {
        self.inner.read().unwrap().soundfont.is_some()
    }

    pub fn get_last_error(&self) -> Option<String> {
        self.inner.read().unwrap().error.clone()
    }

    pub fn set_last_error(&self, err: Option<String>) {
        self.inner.write().unwrap().error = err;
    }

    /// Searches custom settings, project/RTP folders, and standard OS directories for an `.sf2` soundfont file.
    pub fn detect_soundfont(custom_path: Option<&str>, rtp_path: Option<&str>, project_path: Option<&str>) -> Option<PathBuf> {
        if let Some(cp) = custom_path {
            let p = PathBuf::from(cp);
            if p.is_file() {
                return Some(p);
            }
        }

        // Check project path
        if let Some(proj) = project_path {
            let proj_path = Path::new(proj);
            if let Some(sf) = Self::find_sf2_in_dir(proj_path) {
                return Some(sf);
            }
        }

        // Check RTP path
        if let Some(rtp) = rtp_path {
            let rtp_path = Path::new(rtp);
            if let Some(sf) = Self::find_sf2_in_dir(rtp_path) {
                return Some(sf);
            }
        }

        // Search OS directories
        for candidate in Self::system_search_paths() {
            if candidate.is_file() {
                return Some(candidate);
            }
        }

        None
    }

    fn find_sf2_in_dir(dir: &Path) -> Option<PathBuf> {
        if !dir.is_dir() {
            return None;
        }
        let common = ["SoundFont.sf2", "soundfont.sf2", "GeneralUser GS.sf2", "TimGM6mb.sf2", "FluidR3_GM.sf2"];
        for name in common {
            let p = dir.join(name);
            if p.is_file() {
                return Some(p);
            }
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("sf2")).unwrap_or(false) {
                    return Some(p);
                }
            }
        }
        None
    }

    pub fn system_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Linux standard paths
        paths.push(PathBuf::from("/usr/share/sounds/sf2/default-GM.sf2"));
        paths.push(PathBuf::from("/usr/share/sounds/sf2/FluidR3_GM.sf2"));
        paths.push(PathBuf::from("/usr/share/sounds/sf2/TimGM6mb.sf2"));
        paths.push(PathBuf::from("/usr/share/soundfonts/default.sf2"));
        paths.push(PathBuf::from("/usr/share/soundfonts/FluidR3_GM.sf2"));
        paths.push(PathBuf::from("/usr/share/soundfonts/TimGM6mb.sf2"));

        if let Ok(home) = std::env::var("HOME") {
            paths.push(PathBuf::from(format!("{home}/.local/share/soundfonts/default.sf2")));
            paths.push(PathBuf::from(format!("{home}/.local/share/soundfonts/FluidR3_GM.sf2")));
            paths.push(PathBuf::from(format!("{home}/.local/share/soundfonts/TimGM6mb.sf2")));
            // macOS user path
            paths.push(PathBuf::from(format!("{home}/Library/Audio/Sounds/Banks/default.sf2")));
        }

        // macOS system paths
        paths.push(PathBuf::from("/Library/Audio/Sounds/Banks/default.sf2"));
        paths.push(PathBuf::from("/opt/homebrew/share/soundfonts/default.sf2"));

        // Windows AppData paths
        if let Ok(appdata) = std::env::var("APPDATA") {
            paths.push(PathBuf::from(format!("{appdata}/easy-rpg/soundfonts/default.sf2")));
            paths.push(PathBuf::from(format!("{appdata}/easy-rpg/soundfonts/GeneralUser GS.sf2")));
            paths.push(PathBuf::from(format!("{appdata}/easy-rpg/soundfonts/TimGM6mb.sf2")));
        }

        paths
    }
}

/// A single-track audio previewer used by the Sound Test dialog and Resource Manager.
/// Uses `rodio` for all audio files (OGG, MP3, WAV) and synthesizes MIDI via `rustysynth`.
pub struct AudioPlayer {
    _handle: rodio::MixerDeviceSink,
    player: rodio::Player,
    soundfont_manager: SoundFontManager,
}

impl AudioPlayer {
    /// Opens the default audio output device. Returns `None` if audio hardware is unavailable.
    pub fn new(soundfont_manager: SoundFontManager) -> Option<Self> {
        let handle = rodio::DeviceSinkBuilder::open_default_sink().ok()?;
        let player = rodio::Player::connect_new(&handle.mixer());
        Some(Self {
            _handle: handle,
            player,
            soundfont_manager,
        })
    }

    pub fn soundfont_manager(&self) -> &SoundFontManager {
        &self.soundfont_manager
    }

    /// Stops current audio and plays `path`.
    /// Returns `Err(ERR_SOUNDFONT_MISSING)` if `path` is MIDI and no SoundFont is loaded.
    pub fn play_file(&self, path: &Path) -> Result<(), String> {
        self.player.stop();

        if Self::is_midi(path) {
            let soundfont = self
                .soundfont_manager
                .get_soundfont()
                .ok_or_else(|| ERR_SOUNDFONT_MISSING.to_string())?;
            let source = MidiSource::new(path, soundfont)?;
            self.player.append(source);
            self.player.play();
            return Ok(());
        }

        let file = File::open(path).map_err(|e| format!("Couldn't open {}: {e}", path.display()))?;
        let source = rodio::Decoder::try_from(file).map_err(|e| format!("Couldn't decode {}: {e}", path.display()))?;
        self.player.append(source);
        self.player.play();
        Ok(())
    }

    pub fn stop(&self) {
        self.player.stop();
    }

    /// Sets volume (0.0 to 1.0).
    pub fn set_volume(&self, v: f32) {
        self.player.set_volume(v);
    }

    /// Sets playback speed multiplier (0.5 to 1.5).
    pub fn set_speed(&self, v: f32) {
        self.player.set_speed(v);
    }

    /// Checks if a file path is a MIDI file (.mid or .midi).
    pub fn is_midi(path: &Path) -> bool {
        matches!(
            path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref(),
            Some("mid") | Some("midi")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_midi() {
        assert!(AudioPlayer::is_midi(Path::new("music.mid")));
        assert!(AudioPlayer::is_midi(Path::new("music.midi")));
        assert!(AudioPlayer::is_midi(Path::new("MUSIC.MID")));
        assert!(AudioPlayer::is_midi(Path::new("folder/TRACK.MIDI")));
        assert!(!AudioPlayer::is_midi(Path::new("music.mp3")));
        assert!(!AudioPlayer::is_midi(Path::new("music.ogg")));
        assert!(!AudioPlayer::is_midi(Path::new("music.wav")));
        assert!(!AudioPlayer::is_midi(Path::new("no_extension")));
    }

    #[test]
    fn test_soundfont_manager_lifecycle() {
        let manager = SoundFontManager::new();
        assert!(!manager.is_loaded());
        assert!(manager.get_soundfont().is_none());
        assert!(manager.get_path().is_none());

        manager.unload();
        assert!(!manager.is_loaded());
    }

    #[test]
    fn test_detect_soundfont_missing() {
        let detected = SoundFontManager::detect_soundfont(
            Some("non_existent_file_path_12345.sf2"),
            None,
            None,
        );
        assert_ne!(detected, Some(PathBuf::from("non_existent_file_path_12345.sf2")));
    }
}
