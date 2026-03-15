fn main() {
    println!("--- Analyzing Resampled Audio ---");
    let mut reader = hound::WavReader::open("debug_resampled.wav").expect("Could not open wav");
    let spec = reader.spec();
    println!("Spec: {:?}", spec);

    let mut samples: Vec<f32> = Vec::new();

    if spec.sample_format == hound::SampleFormat::Float {
        samples = reader.samples::<f32>().map(|s| s.unwrap()).collect();
    } else {
        println!("Not a float format");
        return;
    }

    if samples.is_empty() {
        println!("No audio data found in file.");
        return;
    }

    let mut sum_squares = 0.0;
    let mut max_val: f32 = 0.0;
    let mut min_val: f32 = 0.0;
    
    for &s in &samples {
        sum_squares += s * s;
        if s > max_val { max_val = s; }
        if s < min_val { min_val = s; }
    }
    
    let rms = (sum_squares / samples.len() as f32).sqrt();
    
    println!("Captured {} mono samples.", samples.len());
    println!("Stats:");
    println!("  RMS: {:.6}", rms);
    println!("  Max Amp: {:.6}", max_val);
    println!("  Min Amp: {:.6}", min_val);
    
    println!("First 20 samples:");
    for i in 0..std::cmp::min(20, samples.len()) {
        println!("  {}: {:.6}", i, samples[i]);
    }
}
