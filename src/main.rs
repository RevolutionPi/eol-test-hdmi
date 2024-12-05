/*
 * SPDX-License-Identifier: GPL-2.0-or-later
 * SPDX-FileCopyrightText: Copyright 2023-2024 KUNBUS GmbH
 */

//! Test video and audio output by showing coloured frames on the framebuffer and playing back a
//! siren over the default ALSA audio device.

use alsa::pcm::{Access, Format, HwParams, State, PCM};
use alsa::{Direction, ValueOr};
use anyhow::{bail, Context};
use clap::Parser;
use framebuffer::{Framebuffer, KdMode};
use std::{thread, time};

mod cli;

const TTY: &str = "/dev/tty1";
const FB: &str = "/dev/fb0";

// how long a frame is displayed in seconds
const FRAME_LENGTH: u64 = 1;
// amount of time one audio sample should play
const AUDIO_LENGTH: u64 = 1;

enum Color {
    Red,
    Green,
    Blue,
}

// `From` doesn't make sense as we're only trying to represent 3 colors
#[allow(clippy::from_over_into)]
impl Into<[u8; 4]> for Color {
    fn into(self) -> [u8; 4] {
        match self {
            Self::Red => 0xFFu32 << 16,
            Self::Green => 0xFFu32 << 8,
            Self::Blue => 0xFFu32,
        }
        .to_le_bytes()
    }
}

#[allow(clippy::from_over_into)]
impl Into<[u8; 2]> for Color {
    fn into(self) -> [u8; 2] {
        match self {
            Self::Red => rgb888_to_rgb565(0xFF, 0, 0),
            Self::Green => rgb888_to_rgb565(0, 0xFF, 0),
            Self::Blue => rgb888_to_rgb565(0, 0, 0xFF),
        }
        .to_le_bytes()
    }
}

/// Convert a RGB888 color to RGB565
const fn rgb888_to_rgb565(red: u8, green: u8, blue: u8) -> u16 {
    let r = (red >> 3) as u16;
    let g = (green >> 2) as u16;
    let b = (blue >> 3) as u16;

    (r << 11) | (g << 5) | b
}

fn frame_set_color(frame: &mut [u8], color: Color, bytespp: u32) {
    match bytespp {
        2 => {
            let color: [u8; 2] = color.into();
            let mut color = color.iter().cycle().peekable();
            assert!(color.peek().is_some());
            frame.fill_with(|| *color.next().expect("BUG: 2-width color is empty"));
        }
        4 => {
            let color: [u8; 4] = color.into();
            let mut color = color.iter().cycle().peekable();
            assert!(color.peek().is_some());
            frame.fill_with(|| *color.next().expect("BUG: 4-width color is empty"));
        }
        other => panic!("{other} bytes per pixel is not supported"),
    }
}

/// Write frame to framebuffer and wait for `FRAME_LENGTH` seconds.
fn frame_write_color(framebuffer: &mut Framebuffer, frame: &mut [u8], color: Color, bytespp: u32) {
    frame_set_color(frame, color, bytespp);
    framebuffer.write_frame(frame);
    thread::sleep(time::Duration::from_secs(FRAME_LENGTH));
}

/// Display 3 frames, each being shown for `FRAME_LENGTH` seconds. The frames will alternate in
/// Red, Green, and Blue.
fn frame() -> anyhow::Result<()> {
    let mut framebuffer =
        Framebuffer::new(FB).with_context(|| format!("Failed to open framebuffer '{FB}'"))?;

    let fb_width = framebuffer.var_screen_info.xres;
    let fb_height = framebuffer.var_screen_info.yres;
    let line_length = framebuffer.fix_screen_info.line_length;
    let bytespp = framebuffer.var_screen_info.bits_per_pixel / 8;

    println!("w: {fb_width}; h: {fb_height}; line_length: {line_length}; bpp: {bytespp}");

    let mut frame = vec![0u8; (line_length * fb_height) as usize];

    //Disable text mode for tty1
    Framebuffer::set_kd_mode_ex(TTY, KdMode::Graphics)
        .with_context(|| format!("Unable to disable text mode on TTY '{TTY}'"))?;

    frame_write_color(&mut framebuffer, &mut frame, Color::Red, bytespp);
    frame_write_color(&mut framebuffer, &mut frame, Color::Green, bytespp);
    frame_write_color(&mut framebuffer, &mut frame, Color::Blue, bytespp);

    //Reenable text mode in current tty
    Framebuffer::set_kd_mode_ex(TTY, KdMode::Text)
        .with_context(|| format!("Unable to enable text mode on TTY '{TTY}'"))?;

    Ok(())
}

// can be improved if needed
fn play_sine_wave(io: &alsa::pcm::IO<i16>, buf: &mut [i16], pitch: f32) -> anyhow::Result<()> {
    for (i, frame) in buf.iter_mut().enumerate() {
        *frame = ((i as f32 * pitch * ::std::f32::consts::PI / 128.0).sin() * 8192.0) as i16
    }

    // Play it back for AUDIO_LENGTH seconds.
    for _ in 0..AUDIO_LENGTH * 44100 / 1024 {
        assert_eq!(
            io.writei(buf)
                .context("Failed to write sine wave value to audio buffer")?,
            1024
        );
    }

    Ok(())
}

// https://docs.rs/alsa/0.7.0/alsa/pcm/index.html
/// Sound a siren over the default ALSA device. The siren consists of 3 tones, each being played
/// back for `AUDIO_LENGTH` seconds.
fn siren() -> anyhow::Result<()> {
    // Open default playback device
    let pcm = PCM::new("default", Direction::Playback, false)
        .context("Failed to open default playback device")?;
    let info = pcm.info().context("Failed to get playback device info")?;
    let name = info
        .get_name()
        .context("Failed to get default playback device name")?;
    println!("Device: {name}");

    // Set hardware parameters: 44100 Hz / Mono / 16 bit
    let hwp = HwParams::any(&pcm).context("Failed to prepare hardware parameters")?;
    hwp.set_channels(1).context("Failed to set channels")?;
    hwp.set_rate(44100, ValueOr::Nearest)
        .context("Failed to set audio rate")?;
    hwp.set_format(Format::s16())
        .context("Failed to set audio format")?;
    hwp.set_access(Access::RWInterleaved)
        .context("Failed to set audio access")?;
    pcm.hw_params(&hwp)
        .context("Failed to set hardware parameters")?;
    let io = pcm.io_i16().context("Failed to get audio IO")?;

    // Make sure we don't start the stream too early
    let hwp = pcm
        .hw_params_current()
        .context("Failed to get current audio hardware parameters")?;
    let swp = pcm
        .sw_params_current()
        .context("Failed to get current audio software parameters")?;
    swp.set_start_threshold(
        hwp.get_buffer_size()
            .context("Failed to get hardware audio buffer size")?,
    )
    .context("Failed to set audio start threshold")?;
    pcm.sw_params(&swp)
        .context("Failed to set audio software parameters")?;

    // Make a sine wave
    let mut buf = [0i16; 1024];

    play_sine_wave(&io, &mut buf, 2.0).context("Failed to play sine wave")?;
    play_sine_wave(&io, &mut buf, 4.0).context("Failed to play sine wave")?;
    play_sine_wave(&io, &mut buf, 6.0).context("Failed to play sine wave")?;

    if pcm.state() != State::Running {
        pcm.start().context("Failed to start audio playback")?
    };
    // Wait for the stream to finish playback.
    pcm.drain().context("Failed to play audio")?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    // the struct itself is not useful yet as no information is passed to the program. when
    // instantiating, however, the help and version flags become available.
    // instantiation is required to parse cli args
    let _ = cli::Args::parse();

    let frame_thread = thread::spawn(|| -> anyhow::Result<()> { frame() });
    let siren_thread = thread::spawn(|| -> anyhow::Result<()> { siren() });

    if let Ok(frame_thread_status) = frame_thread.join() {
        // propagate error that might have happened in the thread
        frame_thread_status.context("Failed to display frames")?;
    } else {
        bail!("Unable to join frame_thread");
    };
    if let Ok(siren_thread_status) = siren_thread.join() {
        siren_thread_status.context("Failed to play sound")?;
    } else {
        bail!("Unable to join siren_thread");
    };

    Ok(())
}
