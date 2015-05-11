// Hound -- A WAV encoding and decoding library in Rust
// Copyright (C) 2015 Ruud van Asseldonk
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3 of the License only.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

//! Hound, a WAV encoding and decoding library.
//!
//! TODO: Add some introductory text here.
//!
//! Examples
//! ========
//!
//! The following example renders a 440 Hz sine wave, and stores it as as a
//! mono wav file with a sample rate of 44.1 kHz and 16 bits per sample.
//!
//! ```
//! use std::f32::consts::PI;
//! use std::i16;
//! use hound;
//!
//! let spec = hound::WavSpec {
//!     channels: 1,
//!     sample_rate: 44100,
//!     bits_per_sample: 16
//! };
//! let mut writer = hound::WavWriter::create("sine.wav", spec).unwrap();
//! for t in (0 .. 44100).map(|x| x as f32 / 44100.0) {
//!     let sample = (t * 440.0 * 2.0 * PI).sin();
//!     let amplitude = i16::MAX as f32;
//!     writer.write_sample((sample * amplitude) as i16).unwrap();
//! }
//! writer.finalize().unwrap();
//! ```
//!
//! The following example computes the RMS (root mean square) of an audio file.
//!
//! ```
//! use hound;
//!
//! let mut reader = hound::WavReader::open("testsamples/pop.wav").unwrap();
//! let (sqr_sum, n) = reader.samples::<i16>()
//!                          .fold((0_f64, 0_u32), |(sqr_sum, n), s| {
//!     let sample = s.unwrap() as f64;
//!     (sqr_sum + sample * sample, n + 1)
//! });
//! println!("RMS is {}", (sqr_sum / n as f64).sqrt());
//! ```

#![warn(missing_docs)]

use std::error;
use std::fmt;
use std::io;
use std::io::Write;
use std::result;
use read::ReadExt;
use write::WriteExt;

mod read;
mod write;

pub use read::{WavReader, WavSamples};
pub use write::WavWriter;

/// A type that can be used to represent audio samples.
pub trait Sample {
    /// Writes the audio sample to the WAVE data chunk.
    fn write<W: io::Write>(self, writer: &mut W, bits: u16) -> io::Result<()>;

    /// Reads the audio sample from the WAVE data chunk.
    fn read<R: io::Read>(reader: &mut R, bits: u16) -> io::Result<Self>;
}

impl Sample for i16 {
    fn write<W: io::Write>(self, writer: &mut W, _bits: u16) -> io::Result<()> {
        writer.write_le_i16(self)
        // TODO: take bits into account.
    }

    fn read<R: io::Read>(reader: &mut R, _bits: u16) -> io::Result<i16> {
        reader.read_le_i16()
        // TODO: take bits into account.
    }
}

/// Specifies properties of the audio data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WavSpec {
    /// The number of channels.
    pub channels: u16,

    /// The number of samples per second.
    ///
    /// A common value is 44100, this is 44.1 kHz which is used for CD audio.
    pub sample_rate: u32,

    /// The number of bits per sample.
    ///
    /// A common value is 16 bits per sample, which is used for CD audio.
    pub bits_per_sample: u16
}

/// The error type for operations on `WavReader` and `WavWriter`.
#[derive(Debug)]
pub enum Error {
    /// An IO error occured in the underlying reader or writer.
    IoError(io::Error),
    /// Ill-formed WAVE data was encountered.
    FormatError(&'static str),
    /// The sample has more bits than the data type of the sample iterator.
    TooWide,
    /// The format is not supported.
    Unsupported
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter)
           -> result::Result<(), fmt::Error> {
        match *self {
            Error::IoError(ref err) => err.fmt(formatter),
            Error::FormatError(reason) => {
                try!(formatter.write_str("Ill-formed WAVE file: "));
                formatter.write_str(reason)
            },
            Error::TooWide => {
                formatter.write_str("The sample has more bits than the data type of the sample iterator.")
            }
            Error::Unsupported => {
                formatter.write_str("The wave format of the file is not supported.")
            }
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IoError(ref err) => err.description(),
            Error::FormatError(reason) => reason,
            Error::TooWide => "the sample has more bits than the data type of the sample iterator",
            Error::Unsupported => "the wave format of the file is not supported"
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::IoError(ref err) => Some(err),
            Error::FormatError(_) => None,
            Error::TooWide => None,
            Error::Unsupported => None
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

/// A type for results generated by Hound where the error type is hard-wired.
pub type Result<T> = result::Result<T, Error>;

#[test]
fn write_read_is_lossless() {
    let mut buffer = io::Cursor::new(Vec::new());
    let write_spec = WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 16
    };

    {
        let mut writer = WavWriter::new(&mut buffer, write_spec);
        for s in (-1024_i16 .. 1024) {
            writer.write_sample(s).unwrap();
        }
        writer.finalize().unwrap();
    }

    {
        buffer.set_position(0);
        let mut reader = WavReader::new(&mut buffer).unwrap();
        assert_eq!(&write_spec, reader.spec());
        for (expected, read) in (-1024_i16 .. 1024).zip(reader.samples()) {
            assert_eq!(expected, read.unwrap());
        }
    }
}
