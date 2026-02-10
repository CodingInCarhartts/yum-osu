use aubio::{Onset, OnsetMode};
use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type as FilterType, Q_BUTTERWORTH_F32};
use rodio::{Decoder, Source};
use std::fs::File;
use std::io::BufReader;

/// Read an audio file and find the times of the kick beats
pub fn gather_beats(path: &str) -> Vec<f64> {
    println!("Loading audio file: {}", path);
    // Open the file
    let file = File::open(path).expect("Failed to open audio file");

    // Create a reader that buffers the file
    let reader = BufReader::new(file);

    // Decode the audio from the reader
    let decoder = Decoder::new(reader).expect("Failed to decode audio");

    // Get the sample rate of the audio
    let sample_rate = decoder.sample_rate();

    // Collect all of the samples from the audio
    let samples: Vec<f32> = decoder.convert_samples().collect();

    // Find the kick beats in the samples
    let beats = detect_kick_beats(&samples, sample_rate);
    beats
}

/// Find the kick beats in a set of samples
fn detect_kick_beats(samples: &[f32], sample_rate: u32) -> Vec<f64> {
    let buffer_size = 1024;
    let hop_size = 512;

    // Lower the cutoff frequency to capture the bass drum more effectively
    let cutoff_freq = 120.0; // Adjust this based on the bass frequency range
    let q_factor = 1.0; // Narrower Q factor for sharper filtering

    // Use a low-pass filter instead of band-pass
    let lowpass_coefficients = Coefficients::<f32>::from_params(
        FilterType::LowPass,
        sample_rate.hz(),
        cutoff_freq.hz(),
        q_factor,
    )
    .unwrap();

    let mut lowpass_filter = DirectForm1::<f32>::new(lowpass_coefficients);

    // Apply the low-pass filter to the samples - use with_capacity for optimization
    let mut filtered_samples = Vec::with_capacity(samples.len());
    for &sample in samples {
        filtered_samples.push(lowpass_filter.run(sample));
    }

    // Use Energy mode instead of RMS (since Rms doesn't exist in your library)
    let mut onset = Onset::new(OnsetMode::Energy, buffer_size, hop_size, sample_rate).unwrap();

    onset.set_threshold(0.4); // Lower the threshold to catch softer bass hits
    onset.set_silence(-60.0); // Adjust for quieter kicks

    // Pre-allocate beats vector with estimated capacity
    let estimated_beats = samples.len() / (hop_size * 2); // Estimate beat count
    let mut beats = Vec::with_capacity(estimated_beats);
    let mut buffer = vec![0.0; buffer_size];
    let mut position = 0;

    while position + buffer_size <= filtered_samples.len() {
        buffer.copy_from_slice(&filtered_samples[position..position + buffer_size]);

        // Check for an onset
        if onset.do_result(&buffer).unwrap() > 0.0 {
            let onset_time = onset.get_last_s();

            // Post-processing: Ignore beats too close together (e.g., less than 150 ms apart)
            if beats.is_empty() || (onset_time as f64 - beats.last().unwrap()) > 0.15 {
                beats.push(onset_time as f64);
            }
        }

        position += hop_size;
    }

    beats
}
