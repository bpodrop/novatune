export function normalizeSamples(samples: Float32Array): Float32Array {
  let peak = 0
  for (const value of samples) {
    const abs = Math.abs(value)
    if (abs > peak) {
      peak = abs
    }
  }

  if (peak === 0) {
    return samples
  }

  const normalized = new Float32Array(samples.length)
  for (let i = 0; i < samples.length; i += 1) {
    normalized[i] = samples[i] / peak
  }

  return normalized
}
