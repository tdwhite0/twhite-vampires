use bevy::prelude::*;
use bevy::audio::{AudioPlayer, PlaybackSettings, Volume};
use bevy::ui::RelativeCursorPosition;
use rand::Rng;
use crate::components::*;
use crate::resources::{SoundAssets, MusicState, MusicTrack};

// C Major Pentatonic frequencies
const C4: f32 = 261.63;
const D4: f32 = 293.66;
const E4: f32 = 329.63;
const G4: f32 = 392.00;
const A4: f32 = 440.00;
const C5: f32 = 523.25;
const E5: f32 = 659.25;
const G5: f32 = 783.99;
const C3: f32 = 130.81;
const E3: f32 = 164.81;
const G3: f32 = 196.00;

const SAMPLE_RATE: u32 = 44100;

// === WAV encoding ===

fn generate_wav(sample_rate: u32, samples: &[i16]) -> Vec<u8> {
    let data_size = (samples.len() * 2) as u32;
    let file_size = 36 + data_size;
    let mut buf = Vec::with_capacity(file_size as usize + 8);

    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");

    // fmt chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    buf.extend_from_slice(&1u16.to_le_bytes()); // PCM
    buf.extend_from_slice(&1u16.to_le_bytes()); // mono
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
    buf.extend_from_slice(&2u16.to_le_bytes()); // block align
    buf.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    // data chunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_size.to_le_bytes());
    for &s in samples {
        buf.extend_from_slice(&s.to_le_bytes());
    }

    buf
}

// === Tone generation ===

fn generate_tone(freq: f32, duration_secs: f32, sample_rate: u32, attack_ms: f32, decay_ms: f32) -> Vec<i16> {
    let num_samples = (duration_secs * sample_rate as f32) as usize;
    let attack_samples = (attack_ms / 1000.0 * sample_rate as f32) as usize;
    let decay_samples = (decay_ms / 1000.0 * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let sine = (2.0 * std::f32::consts::PI * freq * t).sin();

        // Envelope
        let env = if i < attack_samples {
            i as f32 / attack_samples as f32
        } else if i > num_samples.saturating_sub(decay_samples) {
            (num_samples - i) as f32 / decay_samples as f32
        } else {
            1.0
        };

        samples.push((sine * env * 0.8 * i16::MAX as f32) as i16);
    }

    samples
}

fn generate_tone_with_harmonic(
    freq: f32,
    duration_secs: f32,
    sample_rate: u32,
    attack_ms: f32,
    decay_ms: f32,
    harmonic_ratio: f32,
    harmonic_amp: f32,
) -> Vec<i16> {
    let num_samples = (duration_secs * sample_rate as f32) as usize;
    let attack_samples = (attack_ms / 1000.0 * sample_rate as f32) as usize;
    let decay_samples = (decay_ms / 1000.0 * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let sine = (2.0 * std::f32::consts::PI * freq * t).sin();
        let harmonic = (2.0 * std::f32::consts::PI * freq * harmonic_ratio * t).sin() * harmonic_amp;

        let env = if i < attack_samples {
            i as f32 / attack_samples as f32
        } else if i > num_samples.saturating_sub(decay_samples) {
            (num_samples - i) as f32 / decay_samples as f32
        } else {
            1.0
        };

        samples.push(((sine + harmonic) / (1.0 + harmonic_amp) * env * 0.8 * i16::MAX as f32) as i16);
    }

    samples
}

fn generate_arpeggio(
    notes: &[(f32, f32)], // (freq, start_offset_secs)
    total_duration: f32,
    sample_rate: u32,
    attack_ms: f32,
    decay_ms: f32,
) -> Vec<i16> {
    let num_samples = (total_duration * sample_rate as f32) as usize;
    let mut samples = vec![0i32; num_samples];

    for &(freq, offset) in notes {
        let start = (offset * sample_rate as f32) as usize;
        let note_dur = total_duration - offset;
        let note_samples = (note_dur * sample_rate as f32) as usize;
        let attack_s = (attack_ms / 1000.0 * sample_rate as f32) as usize;
        let decay_s = (decay_ms / 1000.0 * sample_rate as f32) as usize;

        for i in 0..note_samples {
            let idx = start + i;
            if idx >= num_samples {
                break;
            }
            let t = i as f32 / sample_rate as f32;
            let sine = (2.0 * std::f32::consts::PI * freq * t).sin();

            let env = if i < attack_s {
                i as f32 / attack_s as f32
            } else if i > note_samples.saturating_sub(decay_s) {
                (note_samples - i) as f32 / decay_s as f32
            } else {
                1.0
            };

            samples[idx] += (sine * env * 0.5 * i16::MAX as f32) as i32;
        }
    }

    samples
        .iter()
        .map(|&s| s.clamp(i16::MIN as i32, i16::MAX as i32) as i16)
        .collect()
}

fn generate_noise_tone(freqs: &[f32], duration_secs: f32, sample_rate: u32, noise_amount: f32) -> Vec<i16> {
    let num_samples = (duration_secs * sample_rate as f32) as usize;
    let decay_samples = num_samples; // full decay
    let mut rng = rand::thread_rng();
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let mut signal = 0.0f32;
        for &f in freqs {
            signal += (2.0 * std::f32::consts::PI * f * t).sin();
        }
        signal /= freqs.len() as f32;

        let noise: f32 = rng.gen_range(-1.0f32..1.0) * noise_amount;
        signal = signal * (1.0 - noise_amount) + noise;

        // Decay envelope
        let env = (num_samples - i) as f32 / decay_samples as f32;
        samples.push((signal * env * 0.8 * i16::MAX as f32) as i16);
    }

    samples
}

// === Sound asset setup ===

pub fn setup_sound_assets(
    mut commands: Commands,
    mut audio_assets: ResMut<Assets<AudioSource>>,
) {
    let pentatonic = [C4, D4, E4, G4, A4];

    // Enemy death: 5 pentatonic variants
    let enemy_death: [Handle<AudioSource>; 5] = std::array::from_fn(|i| {
        let samples = generate_tone(pentatonic[i], 0.08, SAMPLE_RATE, 5.0, 60.0);
        let wav = generate_wav(SAMPLE_RATE, &samples);
        audio_assets.add(AudioSource { bytes: wav.into() })
    });

    // XP gem: 3 high-register shimmery variants
    let xp_freqs = [C5, E5, G5];
    let xp_gem: [Handle<AudioSource>; 3] = std::array::from_fn(|i| {
        let samples = generate_tone_with_harmonic(xp_freqs[i], 0.06, SAMPLE_RATE, 3.0, 45.0, 2.0, 0.3);
        let wav = generate_wav(SAMPLE_RATE, &samples);
        audio_assets.add(AudioSource { bytes: wav.into() })
    });

    // Player damage: low crunch
    let damage_samples = generate_noise_tone(&[C3, E3], 0.15, SAMPLE_RATE, 0.4);
    let player_damage = audio_assets.add(AudioSource {
        bytes: generate_wav(SAMPLE_RATE, &damage_samples).into(),
    });

    // Level up: rising arpeggio
    let level_up_samples = generate_arpeggio(
        &[(C4, 0.0), (E4, 0.15), (G4, 0.30), (C5, 0.45)],
        0.6,
        SAMPLE_RATE,
        5.0,
        100.0,
    );
    let level_up = audio_assets.add(AudioSource {
        bytes: generate_wav(SAMPLE_RATE, &level_up_samples).into(),
    });

    // Game start: bright chord arpeggio
    let game_start_samples = generate_arpeggio(
        &[(C4, 0.0), (E4, 0.12), (G4, 0.24)],
        0.5,
        SAMPLE_RATE,
        5.0,
        150.0,
    );
    let game_start = audio_assets.add(AudioSource {
        bytes: generate_wav(SAMPLE_RATE, &game_start_samples).into(),
    });

    // Game over: descending
    let game_over_samples = generate_arpeggio(
        &[(C4, 0.0), (G3, 0.2), (E3, 0.4), (C3, 0.6)],
        0.8,
        SAMPLE_RATE,
        5.0,
        200.0,
    );
    let game_over = audio_assets.add(AudioSource {
        bytes: generate_wav(SAMPLE_RATE, &game_over_samples).into(),
    });

    // Weapon pickup: two-tone chime
    let pickup_samples = generate_arpeggio(
        &[(C5, 0.0), (E5, 0.1)],
        0.2,
        SAMPLE_RATE,
        3.0,
        80.0,
    );
    let weapon_pickup = audio_assets.add(AudioSource {
        bytes: generate_wav(SAMPLE_RATE, &pickup_samples).into(),
    });

    // Upgrade selected: chord stab
    let upgrade_samples = generate_arpeggio(
        &[(C5, 0.0), (G5, 0.0)],
        0.12,
        SAMPLE_RATE,
        3.0,
        80.0,
    );
    let upgrade_selected = audio_assets.add(AudioSource {
        bytes: generate_wav(SAMPLE_RATE, &upgrade_samples).into(),
    });

    // Weapon-specific fire sounds
    let proj_samples = generate_tone(E5, 0.10, SAMPLE_RATE, 2.0, 70.0);
    let weapon_projectile = audio_assets.add(AudioSource {
        bytes: generate_wav(SAMPLE_RATE, &proj_samples).into(),
    });

    let lightning_samples = generate_noise_tone(&[A4], 0.10, SAMPLE_RATE, 0.3);
    let weapon_lightning = audio_assets.add(AudioSource {
        bytes: generate_wav(SAMPLE_RATE, &lightning_samples).into(),
    });

    let flame_samples = generate_noise_tone(&[G3], 0.10, SAMPLE_RATE, 0.5);
    let weapon_flame = audio_assets.add(AudioSource {
        bytes: generate_wav(SAMPLE_RATE, &flame_samples).into(),
    });

    let boom_samples = generate_tone(D4, 0.10, SAMPLE_RATE, 3.0, 70.0);
    let weapon_boomerang = audio_assets.add(AudioSource {
        bytes: generate_wav(SAMPLE_RATE, &boom_samples).into(),
    });

    commands.insert_resource(SoundAssets {
        enemy_death,
        xp_gem,
        player_damage,
        level_up,
        game_start,
        game_over,
        weapon_pickup,
        upgrade_selected,
        weapon_projectile,
        weapon_lightning,
        weapon_flame,
        weapon_boomerang,
    });
}

// === Background music ===

pub fn setup_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    let tracks = vec![
        MusicTrack {
            name: "Battle",
            handle: asset_server.load("audio/music/battle_loop.ogg"),
        },
        MusicTrack {
            name: "Sprinter",
            handle: asset_server.load("audio/music/pixel_sprinter_loop.ogg"),
        },
        MusicTrack {
            name: "Charge!",
            handle: asset_server.load("audio/music/charge.ogg"),
        },
    ];

    let default_volume = 0.5;

    commands.spawn((
        MusicPlayer,
        AudioPlayer(tracks[0].handle.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: Volume::Linear(default_volume),
            ..default()
        },
    ));

    commands.insert_resource(MusicState {
        selected: 0,
        volume: default_volume,
        muted: false,
        tracks,
    });
}

pub fn start_background_music(
    mut commands: Commands,
    music_state: Res<MusicState>,
    existing: Query<Entity, With<MusicPlayer>>,
) {
    if !existing.is_empty() || music_state.muted {
        return;
    }

    commands.spawn((
        MusicPlayer,
        AudioPlayer(music_state.tracks[music_state.selected].handle.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: Volume::Linear(music_state.volume),
            ..default()
        },
    ));
}

pub fn stop_background_music(
    mut commands: Commands,
    query: Query<Entity, With<MusicPlayer>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn handle_music_track_selection(
    mut commands: Commands,
    mut music_state: ResMut<MusicState>,
    button_query: Query<(&Interaction, &MusicTrackButton), Changed<Interaction>>,
    player_query: Query<Entity, With<MusicPlayer>>,
    panel_query: Query<Entity, With<SettingsPanel>>,
    debug: Res<crate::resources::DebugSettings>,
    weapons: Res<crate::resources::PlayerWeapons>,
    font_state: Res<crate::resources::GameFontState>,
) {
    for (interaction, track_button) in button_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let new_index = track_button.0;
        if new_index == music_state.selected && !music_state.muted {
            return;
        }

        music_state.selected = new_index;
        music_state.muted = false;

        for entity in player_query.iter() {
            commands.entity(entity).despawn();
        }

        commands.spawn((
            MusicPlayer,
            AudioPlayer(music_state.tracks[new_index].handle.clone()),
            PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Loop,
                volume: Volume::Linear(music_state.volume),
                ..default()
            },
        ));

        // Rebuild settings panel to reflect new selection
        for entity in panel_query.iter() {
            commands.entity(entity).despawn();
        }
        super::ui::rebuild_settings_panel(&mut commands, &debug, &weapons, &music_state, &font_state);
    }
}

pub fn handle_music_mute(
    mut commands: Commands,
    mut music_state: ResMut<MusicState>,
    button_query: Query<&Interaction, (Changed<Interaction>, With<MusicMuteButton>)>,
    player_query: Query<Entity, With<MusicPlayer>>,
    panel_query: Query<Entity, With<SettingsPanel>>,
    debug: Res<crate::resources::DebugSettings>,
    weapons: Res<crate::resources::PlayerWeapons>,
    font_state: Res<crate::resources::GameFontState>,
) {
    for interaction in button_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if music_state.muted {
            music_state.muted = false;
            commands.spawn((
                MusicPlayer,
                AudioPlayer(music_state.tracks[music_state.selected].handle.clone()),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Loop,
                    volume: Volume::Linear(music_state.volume),
                    ..default()
                },
            ));
        } else {
            music_state.muted = true;
            for entity in player_query.iter() {
                commands.entity(entity).despawn();
            }
        }

        // Rebuild settings panel
        for entity in panel_query.iter() {
            commands.entity(entity).despawn();
        }
        super::ui::rebuild_settings_panel(&mut commands, &debug, &weapons, &music_state, &font_state);
    }
}

pub fn handle_music_volume_slider(
    mut music_state: ResMut<MusicState>,
    track_query: Query<
        (&Interaction, &RelativeCursorPosition),
        With<MusicVolumeSliderTrack>,
    >,
    mut fill_query: Query<&mut Node, With<MusicVolumeSliderFill>>,
    mut text_query: Query<&mut Text, With<MusicVolumeText>>,
    mut sink_query: Query<&mut AudioSink, With<MusicPlayer>>,
) {
    let Ok((interaction, relative_cursor)) = track_query.single() else {
        return;
    };

    if *interaction != Interaction::Pressed {
        return;
    }

    let Some(pos) = relative_cursor.normalized else {
        return;
    };

    let value = (pos.x + 0.5).clamp(0.0, 1.0);
    music_state.volume = value;

    if let Ok(mut fill_node) = fill_query.single_mut() {
        fill_node.width = Val::Percent(value * 100.0);
    }

    if let Ok(mut text) = text_query.single_mut() {
        **text = format!("{:.0}%", value * 100.0);
    }

    if !music_state.muted {
        if let Ok(mut sink) = sink_query.single_mut() {
            sink.set_volume(Volume::Linear(value));
        }
    }
}
