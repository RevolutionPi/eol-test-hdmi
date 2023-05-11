/*
 * SPDX-License-Identifier: GPL-2.0-or-later
 * SPDX-FileCopyrightText: Copyright 2023 KUNBUS GmbH
 */

//! Test video and audio output by showing coloured frames on the framebuffer and playing back a
//! siren over the default ALSA audio device.

use alsa::pcm::{Access, Format, HwParams, State, PCM};
use alsa::{Direction, ValueOr};
use anyhow::{bail, Context};
use framebuffer::{Framebuffer, KdMode};
use std::{thread, time};

const TTY: &str = "/dev/tty1";
const FB: &str = "/dev/fb0";

// how long a frame is displayed in seconds
const FRAME_LENGTH: u64 = 1;
// amount of time one audio sample should play
const AUDIO_LENGTH: u64 = 1;

fn frame_set_color(frame: &mut [u8], color: (u8, u8, u8)) {
    for pixel in &mut frame.chunks_mut(4) {
        pixel[0] = color.2;
        pixel[1] = color.1;
        pixel[2] = color.0;
        pixel[3] = 0xff;
    }
}

/// Write frame to framebuffer and wait for `FRAME_LENGTH` seconds.
fn frame_write_color(framebuffer: &mut Framebuffer, frame: &mut [u8], color: (u8, u8, u8)) {
    frame_set_color(frame, color);
    framebuffer.write_frame(frame);
    thread::sleep(time::Duration::from_secs(FRAME_LENGTH));
}

/// Display 3 frames, each being shown for `FRAME_LENGTH` seconds. The frames will alternate in
/// Red, Green, and Blue.
fn frame() -> anyhow::Result<()> {
    let mut framebuffer = Framebuffer::new(FB)?;

    let fb_width = framebuffer.var_screen_info.xres;
    let fb_height = framebuffer.var_screen_info.yres;
    let line_length = framebuffer.fix_screen_info.line_length;
    let bytespp = framebuffer.var_screen_info.bits_per_pixel / 8;

    println!("w: {fb_width}; h: {fb_height}; line_length: {line_length}; bpp: {bytespp}");

    let mut frame = vec![0u8; (line_length * fb_height) as usize];

    //Disable text mode for tty1
    Framebuffer::set_kd_mode_ex(TTY, KdMode::Graphics)
        .context("Unable to disable text mode on framebuffer")?;

    frame_write_color(&mut framebuffer, &mut frame, (0xff, 0, 0));
    frame_write_color(&mut framebuffer, &mut frame, (0, 0xff, 0));
    frame_write_color(&mut framebuffer, &mut frame, (0, 0, 0xff));

    //Reenable text mode in current tty
    Framebuffer::set_kd_mode_ex(TTY, KdMode::Text)
        .context("Unable to enable text mode on framebuffer")?;

    Ok(())
}

// can be improved if needed
fn play_sine_wave(io: &alsa::pcm::IO<i16>, buf: &mut [i16], pitch: f32) -> anyhow::Result<()> {
    for (i, frame) in buf.iter_mut().enumerate() {
        *frame = ((i as f32 * pitch * ::std::f32::consts::PI / 128.0).sin() * 8192.0) as i16
    }

    // Play it back for AUDIO_LENGTH seconds.
    for _ in 0..AUDIO_LENGTH * 44100 / 1024 {
        assert_eq!(io.writei(buf)?, 1024);
    }

    Ok(())
}

// https://docs.rs/alsa/0.7.0/alsa/pcm/index.html
/// Sound a siren over the default ALSA device. The siren consists of 3 tones, each being played
/// back for `AUDIO_LENGTH` seconds.
fn siren() -> anyhow::Result<()> {
    // Open default playback device
    let pcm = PCM::new("default", Direction::Playback, false)?;
    let info = pcm.info()?;
    let name = info.get_name()?;
    println!("Device: {name}");

    // Set hardware parameters: 44100 Hz / Mono / 16 bit
    let hwp = HwParams::any(&pcm)?;
    hwp.set_channels(1)?;
    hwp.set_rate(44100, ValueOr::Nearest)?;
    hwp.set_format(Format::s16())?;
    hwp.set_access(Access::RWInterleaved)?;
    pcm.hw_params(&hwp)?;
    let io = pcm.io_i16()?;

    // Make sure we don't start the stream too early
    let hwp = pcm.hw_params_current()?;
    let swp = pcm.sw_params_current()?;
    swp.set_start_threshold(hwp.get_buffer_size()?)?;
    pcm.sw_params(&swp)?;

    // Make a sine wave
    let mut buf = [0i16; 1024];

    play_sine_wave(&io, &mut buf, 2.0)?;
    play_sine_wave(&io, &mut buf, 4.0)?;
    play_sine_wave(&io, &mut buf, 6.0)?;

    if pcm.state() != State::Running {
        pcm.start()?
    };
    // Wait for the stream to finish playback.
    pcm.drain()?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    if let Some(arg) = std::env::args().nth(1) {
        if arg == "--version" {
            let name = env!("CARGO_PKG_NAME");
            let version = env!("CARGO_PKG_VERSION");
            eprintln!("{name}: {version}");
            std::process::exit(0);
        }
    }

    let frame_thread = thread::spawn(|| -> anyhow::Result<()> { frame() });
    let siren_thread = thread::spawn(|| -> anyhow::Result<()> { siren() });

    if let Ok(frame_thread_status) = frame_thread.join() {
        // propagate error that might have happened in the thread
        frame_thread_status?;
    } else {
        bail!("Unable to join frame_thread");
    };
    if let Ok(siren_thread_status) = siren_thread.join() {
        siren_thread_status?;
    } else {
        bail!("Unable to join siren_thread");
    };

    Ok(())
}
