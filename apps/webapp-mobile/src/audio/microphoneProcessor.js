class MicrophoneProcessor extends AudioWorkletProcessor {
  process(inputs, outputs) {
    const input = inputs[0]
    const output = outputs[0]

    if (input && input[0]) {
      const channel = input[0]
      this.port.postMessage(channel.slice())

      if (output && output[0]) {
        output[0].fill(0)
      }
    }

    return true
  }
}

registerProcessor('microphone-processor', MicrophoneProcessor)
