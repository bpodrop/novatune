export interface MicrophoneStream {
  sampleRate: number
  stop: () => void
}

export async function startMicrophoneStream(
  frameSize: number,
  onSamples: (samples: Float32Array) => void,
): Promise<MicrophoneStream> {
  if (!navigator.mediaDevices?.getUserMedia) {
    throw new Error('Microphone API is unavailable in this browser')
  }

  const stream = await navigator.mediaDevices.getUserMedia({
    audio: {
      channelCount: 1,
      echoCancellation: false,
      noiseSuppression: false,
      autoGainControl: false,
    },
  })

  const context = new AudioContext()
  const source = context.createMediaStreamSource(stream)
  await context.audioWorklet.addModule(new URL('./microphoneProcessor.js', import.meta.url))
  const processor = new AudioWorkletNode(context, 'microphone-processor', {
    channelCount: 1,
    numberOfInputs: 1,
    numberOfOutputs: 1,
    outputChannelCount: [1],
    processorOptions: {
      frameSize,
    },
  })
  const silentGain = context.createGain()
  silentGain.gain.value = 0

  processor.port.onmessage = (event) => {
    const samples = event.data
    if (samples instanceof Float32Array) {
      onSamples(samples)
    }
  }

  source.connect(processor)
  processor.connect(silentGain)
  silentGain.connect(context.destination)

  return {
    sampleRate: context.sampleRate,
    stop: () => {
      processor.port.onmessage = null
      processor.disconnect()
      silentGain.disconnect()
      source.disconnect()
      stream.getTracks().forEach((track) => track.stop())
      void context.close()
    },
  }
}
