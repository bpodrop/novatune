export function createSineWave(
  frequencyHz: number,
  sampleRate: number,
  sampleCount: number,
): Float32Array {
  const samples = new Float32Array(sampleCount)
  for (let i = 0; i < sampleCount; i += 1) {
    const t = i / sampleRate
    samples[i] = Math.sin(2 * Math.PI * frequencyHz * t)
  }
  return samples
}
