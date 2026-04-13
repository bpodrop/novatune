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
  const processor = context.createScriptProcessor(frameSize, 1, 1)

  processor.onaudioprocess = (event) => {
    const channel = event.inputBuffer.getChannelData(0)
    onSamples(new Float32Array(channel))
  }

  source.connect(processor)
  processor.connect(context.destination)

  return {
    sampleRate: context.sampleRate,
    stop: () => {
      processor.disconnect()
      source.disconnect()
      stream.getTracks().forEach((track) => track.stop())
      void context.close()
    },
  }
}
